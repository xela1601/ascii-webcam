pub const GSCALE: &str = "   .:-=+*#%@";
pub const CAPTURE_DIR: &str = "captures";
pub const CHAR_ASPECT: f64 = 0.5;

pub struct RenderOutput {
    pub ansi: String,
}

pub fn build_lut(invert: bool) -> [char; 256] {
    let chars: Vec<char> = if invert {
        GSCALE.chars().rev().collect()
    } else {
        GSCALE.chars().collect()
    };
    let n = (chars.len() - 1) as f64;
    let mut lut = [' '; 256];
    for (i, entry) in lut.iter_mut().enumerate() {
        let idx = (i as f64 / 255.0 * n).round() as usize;
        *entry = chars[idx.min(chars.len() - 1)];
    }
    lut
}

#[inline]
pub fn grayscale(r: u8, g: u8, b: u8) -> u8 {
    (0.299_f64.mul_add(r as f64, 0.587_f64.mul_add(g as f64, 0.114 * b as f64))) as u8
}

pub fn render(
    raw: &[u8],
    orig_w: u32,
    orig_h: u32,
    app: &mut super::App,
    term_w: u32,
    term_h: u32,
) -> RenderOutput {
    let transposed = app.transpose || orig_w < orig_h;
    let (frame_w, frame_h, buf_stride) = if transposed {
        (orig_h, orig_w, orig_w)
    } else {
        (orig_w, orig_h, orig_w)
    };

    let art_rows = term_h.saturating_sub(1).max(1);
    let video_aspect = frame_w as f64 / frame_h as f64 / CHAR_ASPECT;
    let term_aspect = term_w as f64 / art_rows as f64;

    let (render_w, render_h, pad_x, pad_y) = if video_aspect > term_aspect {
        let rh = (term_w as f64 / video_aspect).round().max(1.0) as u32;
        (term_w, rh, 0i32, ((art_rows as i32 - rh as i32) / 2).max(0))
    } else {
        let rw = (art_rows as f64 * video_aspect).round().max(1.0) as u32;
        (rw, art_rows, ((term_w as i32 - rw as i32) / 2).max(0), 0i32)
    };

    let scale_x = frame_w as f64 / render_w as f64;
    let scale_y = frame_h as f64 / render_h as f64;

    app.frame_count += 1;
    let elapsed = app.start.elapsed().as_secs_f64();
    let fps = if elapsed > 0.0 {
        (app.frame_count as f64 / elapsed) as u32
    } else {
        0
    };

    app.last_frame_plain.clear();
    let mut ansi =
        String::with_capacity((term_w as u32 * art_rows as u32 * 24 + 256) as usize);

    ansi.push_str("\x1b[?2026h\x1b[H\x1b[2J");

    for y in 0..art_rows {
        let col = if y as i32 >= pad_y && (y as i32 - pad_y) < render_h as i32 {
            pad_x as u32 + 1
        } else {
            1
        };
        ansi.push_str(&format!("\x1b[{};{col}H", y + 1));

        let src_y_img = y as i32 - pad_y;
        if src_y_img >= 0 && (src_y_img as u32) < render_h {
            let src_y =
                ((src_y_img as f64 * scale_y) as u32).min(frame_h.saturating_sub(1));
            for x in 0..render_w {
                let src_x =
                    ((x as f64 * scale_x) as u32).min(frame_w.saturating_sub(1));

                let idx = if transposed {
                    ((x * buf_stride + src_y) * 3) as usize
                } else {
                    let sx = if app.mirror {
                        frame_w - 1 - src_x
                    } else {
                        src_x
                    };
                    (src_y * frame_w + sx) as usize * 3
                };

                let (r, g, b) = (raw[idx], raw[idx + 1], raw[idx + 2]);
                let ch = app.lut[grayscale(r, g, b) as usize];
                if app.bw {
                    ansi.push_str(&format!("\x1b[38;2;255;255;255m{ch}"));
                } else {
                    ansi.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}"));
                }
                app.last_frame_plain.push(ch);
            }
        }
        app.last_frame_plain.push('\n');
    }

    let mirror_label = if app.mirror { "M" } else { " " };
    let invert_label = if app.invert { "I" } else { " " };
    let transp_label = if transposed { "T" } else { " " };
    let bw_label = if app.bw { "B" } else { " " };
    let cam_label = format!("{orig_w}x{orig_h}");

    let fps_text = format!(" {fps:3} FPS ");
    let hud = format!(
        "\x1b[{hud_row};1H\x1b[48;2;30;30;30m\x1b[38;2;180;180;180m\
         {fps_text}\
         ┃ {mirror_label}{invert_label}{transp_label}{bw_label} ┃ {cam_label} -> {render_w}x{render_h} \
         ┃ q:quit  r:inv  m:mir  t:trn  b:bw  c:capture\
         \x1b[K\x1b[0m",
        hud_row = term_h,
    );
    ansi.push_str(&hud);
    ansi.push_str("\x1b[?2026l");

    RenderOutput { ansi }
}
