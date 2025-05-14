use crate::wayland::shm::ShmPool;
use super::drawable::{Drawable, color_blend};


// x and y and topleft corner of rect
pub struct Rectangle {
    x:      usize,
    y:      usize,
    width:  usize,
    height: usize,
    radius: usize,
    color:  u32,
}

impl Rectangle {
    pub fn new(x: usize, y: usize, width: usize, height: usize, radius: usize, color: u32) -> Self {
        Rectangle {x, y, width, height, radius, color}
    }
}

impl Drawable for Rectangle {
    fn draw(&self, shm_pool: &mut ShmPool) {
        for g_row in self.y+self.radius..self.y+self.height-self.radius+1 {
            shm_pool.write(&vec![self.color; self.width], g_row*shm_pool.width+self.x);
        }
        for l_row in 1..self.radius {
            let inner_diff = (((self.radius-1).pow(2) - l_row.pow(2)) as f64).sqrt();
            let outer_diff = ((self.radius.pow(2) - l_row.pow(2)) as f64).sqrt();
            shm_pool.write(&vec![self.color; self.width - (2*(self.radius - inner_diff.floor() as usize-1))], (self.y+self.radius-l_row)*shm_pool.width + self.x + self.radius - inner_diff.floor() as usize-1);
            shm_pool.write(&vec![self.color; self.width - (2*(self.radius - inner_diff.floor() as usize-1))], (self.y+self.height-self.radius+l_row)*shm_pool.width + self.x + self.radius - inner_diff.floor() as usize-1);
            for l_col in inner_diff.floor() as usize+1..outer_diff.ceil() as usize {
                let distance = ((l_row.pow(2) + l_col.pow(2)) as f64).sqrt();
                let offset = (self.y+self.radius-l_row)*shm_pool.width + self.x + self.radius - l_col - 1;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y+self.radius-l_row)*shm_pool.width + self.x + self.width - self.radius + l_col;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y+self.height-self.radius+l_row)*shm_pool.width + self.x + self.radius - l_col - 1;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (self.y+self.height-self.radius+l_row)*shm_pool.width + self.x + self.width - self.radius + l_col;
                shm_pool.write_pixel(color_blend(self.color, shm_pool.read_pixel(offset), distance.fract()), offset as isize);
            }
        }
    }
}

