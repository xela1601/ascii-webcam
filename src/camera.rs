use std::panic;

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

pub fn open() -> Result<Camera, String> {
    let index = CameraIndex::Index(0);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    match panic::catch_unwind(|| Camera::new(index, requested)) {
        Err(_) => Err("Camera access denied. Grant permission in System Settings > Privacy & Security > Camera, then restart your terminal.".into()),
        Ok(Ok(mut cam)) => match cam.open_stream() {
            Ok(()) => Ok(cam),
            Err(e) => Err(format_msg(e)),
        },
        Ok(Err(e)) => Err(format_msg(e)),
    }
}

pub fn format_msg(e: impl std::fmt::Debug) -> String {
    let s = format!("{:#?}", e);
    let hint = if s.contains("No such file") || s.contains("V4L2") {
        "No camera detected. Is one connected?"
    } else if s.contains("Permission") || s.contains("authorization") || s.contains("Not authorized") {
        "Check camera permissions."
    } else {
        ""
    };
    if hint.is_empty() {
        format!("Could not open camera.\n{s}")
    } else {
        format!("Could not open camera.\n{hint}\n{s}")
    }
}
