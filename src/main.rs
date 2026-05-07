mod camera;
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
use nokhwa::pixel_format::RgbFormat;

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
}

struct App {
    invert: bool,
    mirror: bool,
    transpose: bool,
    bw: bool,
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
                KeyCode::Char('c') => {
                    if !app.last_frame_plain.is_empty() {
                        let path = format!(
                            "{}/capture_{}.txt",
                            render::CAPTURE_DIR,
                            app.capture_index
                        );
                        if fs::write(&path, &app.last_frame_plain).is_ok() {
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

    let mut app = App {
        invert: args.invert,
        mirror: args.mirror,
        transpose: args.transpose,
        bw: args.bw,
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
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();
    fs::create_dir_all(render::CAPTURE_DIR).ok();

    loop {
        let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));
        let term_w = term_cols as u32;
        let term_h = term_rows as u32;

        if let Ok(frame) = camera.frame() {
            if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                let raw = decoded.as_raw();
                let (orig_w, orig_h) = (decoded.width(), decoded.height());

                let output = render::render(raw, orig_w, orig_h, &mut app, term_w, term_h);

                write!(stdout, "{}", output.ansi).unwrap();
                stdout.flush().unwrap();
            }
        }

        if !handle_input(&mut app) {
            cleanup();
            return;
        }

        std::thread::sleep(FRAME_SLEEP);
    }
}
