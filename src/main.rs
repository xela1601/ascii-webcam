mod camera;
mod logging;
mod output;
mod render;
mod stream;

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
    logging::init(args.verbose);

    log::info!("ascii-webcam v{} starting", env!("CARGO_PKG_VERSION"));

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
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();
    fs::create_dir_all(&app.capture_dir).ok();

    let streamer = if args.http {
        Some(stream::Stream::start(args.port))
    } else {
        None
    };

    let mut vcam = output::Output::start(args.v4l2_device.as_deref(), args.port);

    loop {
        let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));
        let term_w = term_cols as u32;
        let term_h = term_rows as u32;

        if let Some(frame) = camera.next_frame() {
            let rendered = render::render(&frame.rgb, frame.width, frame.height, &mut app, term_w, term_h);
            write!(stdout, "{}", rendered.ansi).unwrap();
            stdout.flush().unwrap();

            if let Some(ref s) = streamer {
                s.update(&frame.rgb, frame.width, frame.height);
            }

            if app.output_enabled {
                if vcam.is_none() {
                    vcam = output::Output::start(args.v4l2_device.as_deref(), args.port);
                }
                if let Some(ref mut v) = vcam {
                    v.update(&frame.rgb, frame.width, frame.height);
                }
            } else {
                vcam = None;
            }
        }

        if !handle_input(&mut app) {
            cleanup();
            return;
        }

        std::thread::sleep(FRAME_SLEEP);
    }
}
