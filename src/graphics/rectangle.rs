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
        for l_row in 0..radius+1 {
            let x_diff = radius - ((radius*radius - l_row*l_row) as f64).sqrt() as usize;
            self.write(&vec![color; w - (2*x_diff)], (y+radius-l_row)*self.width + x + x_diff);
            self.write(&vec![color; w - (2*x_diff)], (y+h-radius+l_row)*self.width + x + x_diff);
        }
        for g_row in y+radius..y+h-radius {
            self.write(&vec![color; w], g_row*self.width+x);
        }
    }

    pub fn shitty_circle(&mut self, x: usize, y: usize, radius: usize, color: u32) {
        for g_row in y-radius..y+radius+1 { 
            for g_col in x-radius..x+radius+1 { 
                let l_row = y.abs_diff(g_row);
                let l_col = y.abs_diff(g_col);
                let distance = (((l_row*l_row)+(l_col*l_col)) as f64).sqrt();
                let offset = g_row*self.width + g_col;
                if (distance.floor() as usize) < radius {
                    self.write_pixel(color, offset as isize);
                } else if (distance.ceil() as usize) <= radius+1 {
                    dbg!(distance);
                    self.write_pixel(color_blend(color, self.read_pixel(offset), distance.fract()), offset as isize);
                }
            }
        }
    }
}
