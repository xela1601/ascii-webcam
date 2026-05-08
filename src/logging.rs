use std::fs;
use std::path::PathBuf;

use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ascii-webcam")
}

pub fn log_file() -> PathBuf {
    data_dir().join("logs").join("ascii-webcam.log")
}

pub fn capture_dir() -> PathBuf {
    data_dir().join("captures")
}

pub fn init(verbose: bool, has_tty: bool) {
    let base = data_dir();
    fs::create_dir_all(base.join("logs")).ok();
    fs::create_dir_all(base.join("captures")).ok();

    let file_level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // In headless mode, show info+ on stderr for visibility
    let stderr_level = if has_tty {
        LevelFilter::Warn
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            stderr_level,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            file_level,
            Config::default(),
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file())
                .unwrap(),
        ),
    ])
    .ok();
}
