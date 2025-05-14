use crate::wayland::shm::ShmPool;

pub fn color_blend(col1: u32, col2: u32, diff: f64) -> u32 {
    // TODO: Account for alpha channel
    let r1 = (col1 & 0x00ff0000) >> 16;
    let g1 = (col1 & 0x0000ff00) >> 8;
    let b1 = col1 & 0x000000ff;
    let r2 = (col2 & 0x00ff0000) >> 16;
    let g2 = (col2 & 0x0000ff00) >> 8;
    let b2 = col2 & 0x000000ff;
    let r3 = if r1 < r2 {
        r1 + ((r2 - r1) as f64 * diff) as u32
    } else {
        r1 - ((r1 - r2) as f64 * diff) as u32
    };
    let g3 = if g1 < g2 {
        g1 + ((g2 - g1) as f64 * diff) as u32
    } else {
        g1 - ((g1 - g2) as f64 * diff) as u32
    };
    let b3 = if b1 < b2 {
        b1 + ((b2 - b1) as f64 * diff) as u32
    } else {
        b1 - ((b1 - b2) as f64 * diff) as u32
    };
    return 0xff000000 + (r3 << 16) + (g3 << 8) + b3;
}

pub trait Drawable {
    fn draw(&self, shm_pool: &mut ShmPool);
}
