cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod v4l2;
        use v4l2 as backend;
    } else {
        mod http;
        use http as backend;
    }
}

pub struct Output {
    inner: backend::Inner,
}

impl Output {
    pub fn start(device: Option<&str>, port: u16) -> Option<Self> {
        backend::Inner::start(device, port).map(|inner| Output { inner })
    }

    pub fn update(&mut self, rgb: &[u8], width: u32, height: u32) {
        self.inner.update(rgb, width, height);
    }
}
