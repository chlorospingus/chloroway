use crate::wayland::{shm::ShmPool, wl_shm::wl_buffer};
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
    fn update(&mut self) {
        // self.x += 1;
    }

    fn draw(&self, buffer: &wl_buffer, shm_pool: &mut ShmPool) {
        for g_row in self.y+self.radius..self.y+self.height-self.radius+1 {
            shm_pool.write(self.color, g_row*buffer.width as usize+self.x + buffer.offset, self.width);
        }
        for l_row in 1..self.radius {
            let inner_diff = (((self.radius-1).pow(2) - l_row.pow(2)) as f64).sqrt();
            let outer_diff = ((self.radius.pow(2) - l_row.pow(2)) as f64).sqrt();
            shm_pool.write(self.color, (self.y+self.radius-l_row)*buffer.width as usize + self.x + self.radius - inner_diff.floor() as usize-1 + buffer.offset, self.width - (2*(self.radius - inner_diff.floor() as usize-1)));
            shm_pool.write(self.color, (self.y+self.height-self.radius+l_row)*buffer.width as usize + self.x + self.radius - inner_diff.floor() as usize-1 + buffer.offset, self.width - (2*(self.radius - inner_diff.floor() as usize-1)));
            for l_col in inner_diff.floor() as usize+1..outer_diff.ceil() as usize {
                // TODO: handle error from read_pixel
                let distance = ((l_row.pow(2) + l_col.pow(2)) as f64).sqrt();
                let offset = (self.y+self.radius-l_row)*buffer.width as usize + self.x + self.radius - l_col - 1 + buffer.offset;
                shm_pool.write_pixel(color_blend(self.color, 0, distance.fract()), offset);
                let offset = (self.y+self.radius-l_row)*buffer.width as usize + self.x + self.width - self.radius + l_col + buffer.offset;
                shm_pool.write_pixel(color_blend(self.color, 0, distance.fract()), offset);
                let offset = (self.y+self.height-self.radius+l_row)*buffer.width as usize + self.x + self.radius - l_col - 1 + buffer.offset;
                shm_pool.write_pixel(color_blend(self.color, 0, distance.fract()), offset);
                let offset = (self.y+self.height-self.radius+l_row)*buffer.width as usize + self.x + self.width - self.radius + l_col + buffer.offset;
                shm_pool.write_pixel(color_blend(self.color, 0, distance.fract()), offset);
            }
        }
    }
}

impl Into<Box<dyn Drawable>> for Rectangle {
    fn into(self) -> Box<dyn Drawable> {
        Box::new(self)
    }
}
