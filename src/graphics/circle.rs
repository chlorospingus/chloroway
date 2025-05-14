use crate::wayland::shm::ShmPool;
use super::drawable::{Drawable, color_blend};

// x and y are center of circle
pub struct Circle {
    x: usize,
    y: usize,
    radius: usize,
    color: u32,
}

impl Circle {
    pub fn new(x: usize, y: usize, radius: usize, color: u32) -> Self {
        Circle { x, y, radius, color }
    }
}

impl Drawable for Circle {
    fn draw(&self, shm_pool: &mut ShmPool) {
        for l_row in 1..self.radius { 
            let inner_diff = (((self.radius-1).pow(2) - l_row.pow(2)) as f64).sqrt();
            let outer_diff = ((self.radius.pow(2) - l_row.pow(2)) as f64).sqrt();
            let row: Vec<u32> = vec![self.color; 2*(inner_diff.floor() as usize)];
            shm_pool.write(&row, (self.y-l_row)*shm_pool.width + self.x - inner_diff.floor() as usize);
            shm_pool.write(&row, (self.y+l_row-1)*shm_pool.width + self.x - inner_diff.floor() as usize);
            for l_col in (inner_diff.floor() as usize+1)..(outer_diff.ceil() as usize) {
                let distance = ((l_row.pow(2) + l_col.pow(2)) as f64).sqrt();
                let offset = (self.y-l_row)*shm_pool.width + self.x - l_col;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y-l_row)*shm_pool.width + self.x + l_col-1;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y+l_row-1)*shm_pool.width + self.x - l_col;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y+l_row-1)*shm_pool.width + self.x + l_col-1;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
            }
        }
    }
}
