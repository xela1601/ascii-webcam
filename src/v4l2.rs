#![cfg(target_os = "linux")]

use std::fs::File;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;

// Minimal v4l2 definitions for writing to v4l2loopback
const VIDIOC_S_FMT: libc::c_ulong = 0xC0505605;
const V4L2_BUF_TYPE_VIDEO_OUTPUT: u32 = 2;
const V4L2_FIELD_NONE: u32 = 0;
const V4L2_PIX_FMT_RGB24: u32 = 0x33524742; // "RGB3"

#[repr(C)]
#[derive(Default)]
struct V4l2Format {
    typ: u32,
    fmt: V4l2PixFormat,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct V4l2PixFormat {
    width: u32,
    height: u32,
    pixelformat: u32,
    field: u32,
    bytesperline: u32,
    sizeimage: u32,
    colorspace: u32,
    priv_: u32,
    flags: u32,
    ycbcr_enc: u32,
    quantization: u32,
    xfer_func: u32,
}

pub struct Output {
    file: File,
}

impl Output {
    pub fn open(device: &str, width: u32, height: u32) -> io::Result<Self> {
        let file = File::create(device)?;
        let fd = file.as_raw_fd();

        let fmt = V4l2Format {
            typ: V4L2_BUF_TYPE_VIDEO_OUTPUT,
            fmt: V4l2PixFormat {
                width,
                height,
                pixelformat: V4L2_PIX_FMT_RGB24,
                field: V4L2_FIELD_NONE,
                bytesperline: width * 3,
                sizeimage: width * height * 3,
                ..Default::default()
            },
        };

        let ret = unsafe { libc::ioctl(fd, VIDIOC_S_FMT, &fmt) };
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Output { file })
    }

    pub fn write(&mut self, rgb: &[u8]) -> io::Result<()> {
        self.file.write_all(rgb)
    }
}
