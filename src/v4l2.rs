#![cfg(target_os = "linux")]

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;

const VIDIOC_S_FMT: libc::c_ulong = 0xC0505605;
const VIDIOC_STREAMON: libc::c_ulong = 0x40045612;
const V4L2_BUF_TYPE_VIDEO_OUTPUT: u32 = 2;
const V4L2_FIELD_NONE: u32 = 0;
const V4L2_PIX_FMT_RGB24: u32 = 0x33524742;

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
    file: std::fs::File,
    fd: std::os::fd::RawFd,
}

impl Drop for Output {
    fn drop(&mut self) {
        let typ = V4L2_BUF_TYPE_VIDEO_OUTPUT;
        unsafe {
            libc::ioctl(self.fd, 0x40045613, &typ);
        }
    }
}

impl Output {
    pub fn open(device: &str, width: u32, height: u32) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).open(device)?;
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

        if unsafe { libc::ioctl(fd, VIDIOC_S_FMT, &fmt) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let typ = V4L2_BUF_TYPE_VIDEO_OUTPUT;
        if unsafe { libc::ioctl(fd, VIDIOC_STREAMON, &typ) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Output { file, fd })
    }

    pub fn write_frame(&mut self, rgb: &[u8]) -> io::Result<()> {
        self.file.write_all(rgb)
    }
}
