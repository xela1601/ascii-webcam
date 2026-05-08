use std::panic;

use nokhwa::{
    pixel_format::{RgbFormat, YuyvFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

pub struct Frame {
    pub rgb: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct Cam {
    cam: Camera,
    is_yuyv: bool,
}

pub fn open() -> Result<Cam, String> {
    let idx = CameraIndex::Index(0);
    log::info!("opening camera");

    let strategies: [(bool, RequestedFormatType); 4] = [
        (false, RequestedFormatType::None),
        (false, RequestedFormatType::AbsoluteHighestResolution),
        (false, RequestedFormatType::AbsoluteHighestFrameRate),
        (true, RequestedFormatType::None),
    ];

    let mut last_err = String::new();
    for &(yuyv, fmt_type) in &strategies {
        let label = if yuyv { "YUYV" } else { "RGB" };
        log::debug!("trying camera format {} ({:?})", label, fmt_type);
        if yuyv {
            let req = RequestedFormat::new::<YuyvFormat>(fmt_type);
            match try_open(idx.clone(), req) {
                Ok(cam) => {
                    log::info!("camera opened (YUYV)");
                    return Ok(Cam { cam, is_yuyv: true });
                }
                Err(e) => last_err = e,
            }
        } else {
            let req = RequestedFormat::new::<RgbFormat>(fmt_type);
            match try_open(idx.clone(), req) {
                Ok(cam) => {
                    log::info!("camera opened (RGB)");
                    return Ok(Cam { cam, is_yuyv: false });
                }
                Err(e) => last_err = e,
            }
        }
    }

    log::error!("camera open failed: {}", last_err);
    Err(last_err)
}

fn try_open(
    idx: CameraIndex,
    requested: impl Into<nokhwa::utils::RequestedFormat<'static>>,
) -> Result<Camera, String> {
    let req: nokhwa::utils::RequestedFormat = requested.into();
    match panic::catch_unwind(|| Camera::new(idx, req)) {
        Err(_) => Err("Camera access denied. Grant permission in System Settings > Privacy & Security > Camera, then restart your terminal.".into()),
        Ok(Ok(mut cam)) => match cam.open_stream() {
            Ok(()) => Ok(cam),
            Err(e) => Err(format_msg(e)),
        },
        Ok(Err(e)) => Err(format_msg(e)),
    }
}

impl Cam {
    pub fn next_frame(&mut self) -> Option<Frame> {
        let buffer = self.cam.frame().ok()?;
        if self.is_yuyv {
            let w = buffer.resolution().width();
            let h = buffer.resolution().height();
            let rgb = yuyv_to_rgb(buffer.buffer(), w, h);
            Some(Frame { rgb, width: w, height: h })
        } else {
            let decoded = buffer.decode_image::<RgbFormat>().ok()?;
            let width = decoded.width();
            let height = decoded.height();
            Some(Frame { rgb: decoded.into_raw(), width, height })
        }
    }
}

fn yuyv_to_rgb(yuv: &[u8], w: u32, h: u32) -> Vec<u8> {
    let len = (w * h * 3) as usize;
    let mut rgb = vec![0u8; len];
    for y in 0..h {
        for x in (0..w).step_by(2) {
            let i = ((y * w + x) * 2) as usize;
            let y0 = yuv[i] as f64;
            let u  = yuv[i + 1] as f64 - 128.0;
            let y1 = yuv[i + 2] as f64;
            let v  = yuv[i + 3] as f64 - 128.0;

            let out0 = ((y * w + x) * 3) as usize;
            rgb[out0]     = (y0 + 1.402 * v).clamp(0.0, 255.0) as u8;
            rgb[out0 + 1] = (y0 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
            rgb[out0 + 2] = (y0 + 1.772 * u).clamp(0.0, 255.0) as u8;

            let out1 = ((y * w + x + 1) * 3) as usize;
            rgb[out1]     = (y1 + 1.402 * v).clamp(0.0, 255.0) as u8;
            rgb[out1 + 1] = (y1 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
            rgb[out1 + 2] = (y1 + 1.772 * u).clamp(0.0, 255.0) as u8;
        }
    }
    rgb
}

pub fn format_msg(e: impl std::fmt::Debug) -> String {
    let s = format!("{:#?}", e);
    let hint = if s.contains("No such file") || s.contains("V4L2") {
        "No camera detected. Is one connected?"
    } else if s.contains("Permission") || s.contains("authorization") || s.contains("Not authorized") {
        "Check camera permissions."
    } else if s.contains("lockForConfiguration") || s.contains("Lock Rejected") {
        "Camera is locked by another app. Close other apps using the camera (FaceTime, Zoom, Photo Booth, etc.) and try again."
    } else {
        ""
    };
    if hint.is_empty() {
        format!("Could not open camera.\n{s}")
    } else {
        format!("Could not open camera.\n{hint}\n{s}")
    }
}
