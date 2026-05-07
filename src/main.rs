use std::fs;
use std::io::{self, Write};
use std::panic;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self},
};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

const GSCALE: &str = "   .:-=+*#%@";
const CAPTURE_DIR: &str = "captures";

#[derive(Parser)]
#[command(
    version,
    about = "A terminal-based webcam that renders your live feed as colored ASCII art"
)]
struct Args {
    #[arg(short = 'r', long, help = "Invert brightness")]
    invert: bool,
    #[arg(short = 'm', long, help = "Mirror horizontally")]
    mirror: bool,
}

fn build_lut(invert: bool) -> [char; 256] {
    let chars: Vec<char> = if invert {
        GSCALE.chars().rev().collect()
    } else {
        GSCALE.chars().collect()
    };
    let n = (chars.len() - 1) as f64;
    let mut lut = [' '; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let idx = (i as f64 / 255.0 * n).round() as usize;
        *entry = chars[idx.min(chars.len() - 1)];
    }
    lut
}

#[inline]
fn grayscale(r: u8, g: u8, b: u8) -> u8 {
    (0.299_f64.mul_add(r as f64, 0.587_f64.mul_add(g as f64, 0.114 * b as f64))) as u8
}

fn cleanup() {
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).ok();
    terminal::disable_raw_mode().ok();
}

fn try_open_camera() -> Result<Camera, String> {
    let index = CameraIndex::Index(0);

    let strategies = [
        RequestedFormatType::AbsoluteHighestResolution,
        RequestedFormatType::None,
        RequestedFormatType::AbsoluteHighestFrameRate,
    ];

    let mut last_err = String::new();
    for &strategy in &strategies {
        let requested = RequestedFormat::new::<RgbFormat>(strategy);
        match panic::catch_unwind(|| Camera::new(index.clone(), requested)) {
            Err(_) => {
                return Err("Camera access denied. Grant permission in System Settings > Privacy & Security > Camera, then restart your terminal.".into());
            }
            Ok(Ok(mut cam)) => match cam.open_stream() {
                Ok(()) => return Ok(cam),
                Err(e) => last_err = format!("{:#?}", e),
            },
            Ok(Err(e)) => last_err = format!("{:#?}", e),
        }
    }

    let hint = if last_err.contains("Permission denied")
        || last_err.contains("authorization")
        || last_err.contains("Not authorized")
    {
        "Check camera permissions."
    } else if last_err.contains("No such file") || last_err.contains("V4L2") {
        "No camera detected. Is one connected?"
    } else {
        ""
    };

    Err(format!("Could not open camera. {hint}\n{last_err}"))
}

fn main() {
    let args = Args::parse();

    let mut invert = args.invert;
    let mut mirror = args.mirror;
    let mut lut = build_lut(invert);

    let mut camera = match try_open_camera() {
        Ok(cam) => cam,
        Err(msg) => {
            eprintln!("Error: {msg}");
            std::process::exit(1);
        }
    };

    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let start = Instant::now();
    let mut frames: u64 = 0;
    let mut capture_count: u32 = 0;

    fs::create_dir_all(CAPTURE_DIR).ok();

    let mut ascii_plain = String::new();

    loop {
        let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));
        let tw = term_cols as u32;
        let th = term_rows as u32;

        if let Ok(frame) = camera.frame() {
            if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                let raw = decoded.as_raw();
                let (orig_w, orig_h) = (decoded.width(), decoded.height());

                let cam_label = format!("{orig_w}x{orig_h}");

                // Auto-rotate portrait cameras to landscape
                let rotated = orig_w < orig_h;
                let (fw, fh) = if rotated {
                    (orig_h, orig_w)
                } else {
                    (orig_w, orig_h)
                };

                let art_rows = th.saturating_sub(1).max(1);
                let video_aspect = fw as f64 / fh as f64;
                let term_aspect = tw as f64 / art_rows as f64;

                // Fit full video within terminal, preserving aspect ratio.
                // The render area is the largest rectangle inside the terminal
                // that has the same shape as the camera frame — no cropping.
                let (rw, rh, ox, oy) = if video_aspect > term_aspect {
                    let rh = (tw as f64 / video_aspect).round().max(1.0) as u32;
                    (tw, rh, 0i32, ((art_rows as i32 - rh as i32) / 2).max(0))
                } else {
                    let rw = (art_rows as f64 * video_aspect).round().max(1.0) as u32;
                    (rw, art_rows, ((tw as i32 - rw as i32) / 2).max(0), 0i32)
                };

                let scale_x = fw as f64 / rw as f64;
                let scale_y = fh as f64 / rh as f64;

                frames += 1;
                let elapsed = start.elapsed().as_secs_f64();
                let fps = if elapsed > 0.0 {
                    (frames as f64 / elapsed) as u32
                } else {
                    0
                };

                ascii_plain.clear();
                let mut buf = String::with_capacity(
                    (rw as u32 * rh as u32 * 24 + rh as u32 * 10 + 256) as usize,
                );

                buf.push_str("\x1b[?2026h\x1b[H\x1b[2J");

                for y in 0..art_rows {
                    let col = if y as i32 >= oy && (y as i32 - oy) < rh as i32 {
                        ox as u32 + 1
                    } else {
                        1
                    };
                    buf.push_str(&format!("\x1b[{};{col}H", y + 1));

                    let sy = y as i32 - oy;
                    if sy >= 0 && (sy as u32) < rh {
                        let src_y = ((sy as f64 * scale_y) as u32).min(fh.saturating_sub(1));
                        for x in 0..rw {
                            let src_x = ((x as f64 * scale_x) as u32).min(fw.saturating_sub(1));
                            let sx = if mirror { fw - 1 - src_x } else { src_x };
                            let idx = if rotated {
                                // 90° CW rotation; mirror flips the new x-axis
                                let ry = if mirror { orig_h - 1 - src_x } else { src_x };
                                ((ry as u32 * orig_w + (orig_w - 1 - src_y as u32)) * 3) as usize
                            } else {
                                (src_y as u32 * fw + sx) as usize * 3
                            };
                            let r = raw[idx];
                            let g = raw[idx + 1];
                            let b = raw[idx + 2];
                            let gray = grayscale(r, g, b);
                            let ch = lut[gray as usize];
                            buf.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}"));
                            ascii_plain.push(ch);
                        }
                    }
                    ascii_plain.push('\n');
                }

                // HUD bar
                let mirror_label = if mirror { "M" } else { " " };
                let invert_label = if invert { "I" } else { " " };
                let fps_text = format!(" {fps:3} FPS ");
                let hud = format!(
                    "\x1b[{hud_row};1H\x1b[48;2;30;30;30m\x1b[38;2;180;180;180m\
                     {fps_text}\
                     ┃ {mirror_label}{invert_label} ┃ {cam_label} -> {rw}x{rh} \
                     ┃ q:quit  r:inv  m:mir  c:capture\
                     \x1b[K\x1b[0m",
                    hud_row = th,
                );

                buf.push_str(&hud);
                buf.push_str("\x1b[?2026l");

                write!(stdout, "{buf}").unwrap();
                stdout.flush().unwrap();
            }
        }

        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        cleanup();
                        return;
                    }
                    KeyCode::Char('q') => {
                        cleanup();
                        return;
                    }
                    KeyCode::Char('r') => {
                        invert = !invert;
                        lut = build_lut(invert);
                    }
                    KeyCode::Char('m') => {
                        mirror = !mirror;
                    }
                    KeyCode::Char('c') => {
                        if !ascii_plain.is_empty() {
                            let path = format!("{}/capture_{}.txt", CAPTURE_DIR, capture_count);
                            if fs::write(&path, &ascii_plain).is_ok() {
                                capture_count += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        std::thread::sleep(Duration::from_micros(100));
    }
}
