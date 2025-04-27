
use crate::wayland::shm::ShmPool;

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
    
    // x and y are center of circle
    pub fn circle(&mut self, x: usize, y: usize, radius: usize, color: u32) {
        for l_row in 0..radius { 
            let x_diff = ((radius*radius - l_row*l_row) as f64).sqrt();
            let mut row: Vec<u32> = vec![color; 2*x_diff as usize];
            self.write(&row, (y+l_row)*self.width + x - x_diff as usize);
            self.write(&row, (y-l_row)*self.width + x - x_diff as usize);
        }
    }
}
