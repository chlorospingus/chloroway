use crate::wayland::{shm::ShmPool, wl_shm::wl_buffer};

pub fn premultiply(color: u32) -> u32 {
    let a = (color & 0xff000000) >> 24;
    let r = (color & 0x00ff0000) >> 16;
    let g = (color & 0x0000ff00) >> 8;
    let b = (color & 0x000000ff) >> 0;
    let alpha = (a as f64 / 0xff as f64);
    (a << 24) +
    (((r as f64*alpha) as u32) << 16) +
    (((g as f64*alpha) as u32) << 8) +
    (((b as f64*alpha) as u32) << 0)
}

pub fn color_blend(col1: u32, col2: u32, diff: f64) -> u32 {
    let a1 = (col1 & 0xff000000) >> 24;
    let r1 = (col1 & 0x00ff0000) >> 16;
    let g1 = (col1 & 0x0000ff00) >> 8;
    let b1 = (col1 & 0x000000ff) >> 0;
    let a2 = (col2 & 0xff000000) >> 24;
    let r2 = (col2 & 0x00ff0000) >> 16;
    let g2 = (col2 & 0x0000ff00) >> 8;
    let b2 = (col2 & 0x000000ff) >> 0;
    (((a1 as f64 + (a2 as f64 - a1 as f64) * diff).round() as u32) << 24) +
    (((r1 as f64 + (r2 as f64 - r1 as f64) * diff).round() as u32) << 16) +
    (((g1 as f64 + (b2 as f64 - g1 as f64) * diff).round() as u32) << 8 ) +
    (((b1 as f64 + (g2 as f64 - b1 as f64) * diff).round() as u32) << 0 )
}

pub fn color_over(over: u32, under: u32) -> u32 {
    let a_over  = (over  & 0xff000000) >> 24;
    let a_under = (under & 0xff000000) >> 24;
    if a_over == 0xff || a_under == 0 {
        return over;
    }
    if a_over == 0 {
        return under;
    }
    let r_over  = (over  & 0x00ff0000) >> 16;
    let r_under = (under & 0x00ff0000) >> 16;
    let g_over  = (over  & 0x0000ff00) >> 8;
    let g_under = (under & 0x0000ff00) >> 8;
    let b_over  = (over  & 0x000000ff) >> 0;
    let b_under = (under & 0x000000ff) >> 0;
    ((a_over + (a_under as f64 * (0xff - a_over) as f64 / 0xff as f64) as u32).min(0xff) << 24).min(0xff) +
    ((r_over + (r_under as f64 * (0xff - a_over) as f64 / 0xff as f64) as u32).min(0xff) << 16).min(0xff) +
    ((g_over + (g_under as f64 * (0xff - a_over) as f64 / 0xff as f64) as u32).min(0xff) << 8) +
    ((b_over + (b_under as f64 * (0xff - a_over) as f64 / 0xff as f64) as u32).min(0xff) << 0)
}

pub trait Drawable : Send {
    fn update(&mut self);
    fn draw(&self, buffer: &wl_buffer, shm_pool: &mut ShmPool);
}
