use std::usize;

use crate::wayland::shm::ShmPool;

fn color_blend(col1: u32, col2: u32, diff: f64) -> u32 {
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

// l_row means local row and g_row means global row
impl ShmPool {

    // x and y are topleft corner of rect
    pub fn rectangle(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        for g_row in y..y+h {
            self.write(&vec![color; w], g_row*self.width+x);
        }
    }

    // x and y and topleft corner of rect
    pub fn rounded_rectangle(
        &mut self,
        x:      usize,
        y:      usize,
        w:      usize,
        h:      usize,
        radius: usize,
        color:  u32
    ) {
        for l_row in 1..radius {
            let inner_diff = (((radius-1).pow(2) - l_row.pow(2)) as f64).sqrt();
            let outer_diff = ((radius.pow(2) - l_row.pow(2)) as f64).sqrt();
            self.write(&vec![color; w - (2*(radius - inner_diff.floor() as usize-1))], (y+radius-l_row)*self.width + x + radius - inner_diff.floor() as usize-1);
            self.write(&vec![color; w - (2*(radius - inner_diff.floor() as usize-1))], (y+h-radius+l_row)*self.width + x + radius - inner_diff.floor() as usize-1);
            for l_col in inner_diff.floor() as usize+1..outer_diff.ceil() as usize {
                let distance = ((l_row.pow(2) + l_col.pow(2)) as f64).sqrt();
                let offset = (y+radius-l_row)*self.width + x + radius - l_col - 1;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y+radius-l_row)*self.width + x + w - radius + l_col;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y+h-radius+l_row)*self.width + x + radius - l_col - 1;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y+h-radius+l_row)*self.width + x + w - radius + l_col;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
            }
        }
        for g_row in y+radius..y+h-radius+1 {
            self.write(&vec![color; w], g_row*self.width+x);
        }
    }
    
    // x and y are center of circle
    pub fn circle(&mut self, x: usize, y: usize, radius: usize, color: u32) {
        for l_row in 1..radius { 
            let inner_diff = (((radius-1).pow(2) - l_row.pow(2)) as f64).sqrt();
            let outer_diff = ((radius.pow(2) - l_row.pow(2)) as f64).sqrt();
            let row: Vec<u32> = vec![color; 2*(inner_diff.floor() as usize)];
            self.write(&row, (y-l_row)*self.width + x - inner_diff.floor() as usize);
            self.write(&row, (y+l_row-1)*self.width + x - inner_diff.floor() as usize);
            for l_col in (inner_diff.floor() as usize+1)..(outer_diff.ceil() as usize) {
                let distance = ((l_row.pow(2) + l_col.pow(2)) as f64).sqrt();
                let offset = (y-l_row)*self.width + x - l_col;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y-l_row)*self.width + x + l_col-1;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y+l_row-1)*self.width + x - l_col;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                let offset = (y+l_row-1)*self.width + x + l_col-1;
                self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
            }
        }
    }
}
