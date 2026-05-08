use std::io::{Cursor, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use image::ImageEncoder;

pub struct Inner {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Inner {
    pub fn start(port: u16) -> Option<Self> {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buf = buffer.clone();

        thread::spawn(move || {
            let addr = format!("0.0.0.0:{port}");
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    log::error!("HTTP stream: cannot bind {addr}: {e}");
                    return;
                }
            };
            log::info!("HTTP MJPEG stream at http://localhost:{port}");

            for stream in listener.incoming() {
                let mut s = match stream { Ok(s) => s, Err(_) => continue };
                let boundary = "ascii-webcam";

                write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary={boundary}\r\n\r\n--{boundary}\r\n"
                ).ok();

                loop {
                    let data = buf.lock().unwrap().clone();
                    if data.is_empty() {
                        thread::sleep(std::time::Duration::from_millis(30));
                        continue;
                    }
                    if write!(s, "Content-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", data.len()).is_err()
                        || s.write_all(&data).is_err()
                        || write!(s, "\r\n--{boundary}\r\n").is_err()
                    {
                        break;
                    }
                }
            }
        });

        Some(Inner { buffer })
    }

    pub fn update(&mut self, rgb: &[u8], width: u32, height: u32) {
        if let Some(img) = image::RgbImage::from_raw(width, height, rgb.to_vec()) {
            let mut buf = Cursor::new(Vec::new());
            if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50)
                .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgb8)
                .is_ok()
            {
                *self.buffer.lock().unwrap() = buf.into_inner();
            }
        }
    }
}
