mod http;

#[cfg(target_os = "linux")]
mod v4l2;

enum Kind {
    Http(http::Inner),
    #[cfg(target_os = "linux")]
    V4l2(v4l2::Inner),
}

pub struct Output(Kind);

impl Output {
    pub fn http(port: u16) -> Option<Self> {
        http::Inner::start(port).map(|i| Output(Kind::Http(i)))
    }

    #[cfg(target_os = "linux")]
    pub fn v4l2(device: Option<&str>) -> Option<Self> {
        v4l2::Inner::start(device).map(|i| Output(Kind::V4l2(i)))
    }

    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)]
    pub fn v4l2(_device: Option<&str>) -> Option<Self> {
        None
    }

    pub fn update(&mut self, rgb: &[u8], width: u32, height: u32) {
        match &mut self.0 {
            Kind::Http(i) => i.update(rgb, width, height),
            #[cfg(target_os = "linux")]
            Kind::V4l2(i) => i.update(rgb, width, height),
        }
    }
}
