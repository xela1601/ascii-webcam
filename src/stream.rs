use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use image::ImageEncoder;

pub struct Stream {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl Stream {
    pub fn start(port: u16) -> Self {
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
            log::info!("HTTP stream: http://localhost:{port}");

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(s) => {
                        log::debug!("HTTP stream: client connected");
                        s
                    }
                    Err(e) => {
                        log::warn!("HTTP stream: connection error: {e}");
                        continue;
                    }
                };

                let boundary = "ascii-webcam-boundary";
                let header = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: multipart/x-mixed-replace; boundary={boundary}\r\n\
                     Connection: close\r\n\
                     Cache-Control: no-cache\r\n\r\n\
                     --{boundary}\r\n"
                );

                if stream.write_all(header.as_bytes()).is_err() {
                    continue;
                }

                loop {
                    let data = buf.lock().unwrap().clone();
                    if data.is_empty() {
                        thread::sleep(std::time::Duration::from_millis(30));
                        continue;
                    }

                    let part = format!(
                        "Content-Type: image/jpeg\r\n\
                         Content-Length: {}\r\n\r\n",
                        data.len()
                    );
                    if stream.write_all(part.as_bytes()).is_err()
                        || stream.write_all(&data).is_err()
                        || stream
                            .write_all(format!("\r\n--{boundary}\r\n").as_bytes())
                            .is_err()
                    {
                        log::debug!("HTTP stream: client disconnected");
                        break;
                    }
                }
            }
        });

        Stream { inner: buffer }
    }

    pub fn update(&self, rgb: &[u8], width: u32, height: u32) {
        if let Some(img) = image::RgbImage::from_raw(width, height, rgb.to_vec()) {
            let mut buf = std::io::Cursor::new(Vec::new());
            if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50)
                .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgb8)
                .is_ok()
            {
                *self.inner.lock().unwrap() = buf.into_inner();
            }
        }
    }
}
