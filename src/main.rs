mod camera;
mod logging;
mod output;
mod render;

use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self},
};

const FRAME_SLEEP: Duration = Duration::from_micros(100);

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
    #[arg(short = 't', long, help = "Transpose raw buffer (swap width/height)")]
    transpose: bool,
    #[arg(short = 'b', long, help = "Black & white mode")]
    bw: bool,
    #[arg(long = "http", help = "HTTP MJPEG stream (macOS virtual camera bridge)")]
    http: bool,
    #[arg(short = 'p', long, help = "HTTP stream port", default_value = "8080")]
    port: u16,
    #[arg(long = "v4l2", help = "v4l2loopback device (Linux virtual camera)")]
    v4l2_device: Option<String>,
    #[arg(short = 'v', long, help = "Verbose logging (debug level to file)")]
    verbose: bool,
}

struct App {
    invert: bool,
    mirror: bool,
    transpose: bool,
    bw: bool,
    output_enabled: bool,
    capture_dir: std::path::PathBuf,
    lut: [char; 256],
    frame_count: u64,
    capture_index: u32,
    last_frame_plain: String,
    start: Instant,
}

fn cleanup() {
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).ok();
    terminal::disable_raw_mode().ok();
}

fn handle_input(app: &mut App) -> bool {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return false;
                }
                KeyCode::Char('q') => return false,
                KeyCode::Char('r') => {
                    app.invert = !app.invert;
                    app.lut = render::build_lut(app.invert);
                }
                KeyCode::Char('m') => app.mirror = !app.mirror,
                KeyCode::Char('t') => app.transpose = !app.transpose,
                KeyCode::Char('b') => app.bw = !app.bw,
                KeyCode::Char('s') => app.output_enabled = !app.output_enabled,
                KeyCode::Char('c') => {
                    if !app.last_frame_plain.is_empty() {
                        let path = app.capture_dir.join(format!("capture_{}.txt", app.capture_index));
                        if fs::write(&path, &app.last_frame_plain).is_ok() {
                            log::info!("capture saved to {}", path.display());
                            app.capture_index += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    true
}

fn main() {
    let args = Args::parse();

    let mut stdout = io::stdout();
    let has_tty = execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).is_ok()
        && terminal::enable_raw_mode().is_ok();
    if !has_tty {
        terminal::disable_raw_mode().ok();
    }

    logging::init(args.verbose, has_tty);
    log::info!("ascii-webcam v{} starting", env!("CARGO_PKG_VERSION"));

    if !has_tty {
        log::info!("no TTY — running headless, press Ctrl+C to stop");
    }

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        log::info!("shutting down...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .ok();

    let mut app = App {
        invert: args.invert,
        mirror: args.mirror,
        transpose: args.transpose,
        bw: args.bw,
        output_enabled: args.http || args.v4l2_device.is_some(),
        capture_dir: logging::capture_dir(),
        lut: render::build_lut(args.invert),
        frame_count: 0,
        capture_index: 0,
        last_frame_plain: String::new(),
        start: Instant::now(),
    };

    let mut camera = match camera::open() {
        Ok(cam) => cam,
        Err(msg) => {
            eprintln!("Error: {msg}");
            std::process::exit(1);
        }
    };

    let mut stdout = io::stdout();
    fs::create_dir_all(&app.capture_dir).ok();

    let mut http_out = if args.http {
        output::Output::http(args.port)
    } else {
        None
    };

    let mut vcam = output::Output::v4l2(args.v4l2_device.as_deref());

    loop {
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            cleanup();
            log::info!("shut down cleanly");
            return;
        }

        let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));
        let term_w = term_cols as u32;
        let term_h = term_rows as u32;

        if let Some(frame) = camera.next_frame() {
            // Periodic status in headless mode (every ~60 frames)
            if !has_tty {
                app.frame_count += 1;
                if app.frame_count % 60 == 0 {
                    let elapsed = app.start.elapsed().as_secs_f64();
                    let fps = if elapsed > 0.0 {
                        (app.frame_count as f64 / elapsed) as u32
                    } else {
                        0
                    };
                    log::info!(
                        "frame {:>6} | {fps:>3} FPS | {}x{}",
                        app.frame_count,
                        frame.width,
                        frame.height
                    );
                }
            }

            if has_tty {
                let rendered = render::render(&frame.rgb, frame.width, frame.height, &mut app, term_w, term_h);
                write!(stdout, "{}", rendered.ansi).unwrap();
                stdout.flush().unwrap();

                if let Some(ref mut s) = http_out {
                    s.update(&rendered.image, rendered.image_w, rendered.image_h);
                }
            } else {
                // Headless: render ASCII to image for the stream
                let rendered = render::render(&frame.rgb, frame.width, frame.height, &mut app, term_w, term_h);

                if let Some(ref mut s) = http_out {
                    s.update(&rendered.image, rendered.image_w, rendered.image_h);
                }
            }

            if app.output_enabled {
                if vcam.is_none() {
                    vcam = output::Output::v4l2(args.v4l2_device.as_deref());
                }
                if let Some(ref mut v) = vcam {
                    v.update(&frame.rgb, frame.width, frame.height);
                }
            } else {
                vcam = None;
            }
        }

        if has_tty {
            if !handle_input(&mut app) {
                cleanup();
                return;
            }
        }

        std::thread::sleep(FRAME_SLEEP);
    }
}
