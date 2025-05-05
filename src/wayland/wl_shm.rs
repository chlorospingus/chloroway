use std::{error::Error, io::Write, os::unix::net::SocketAncillary, u8};
use crate::wayland::{shm, surface::UnsetErr, vec_utils::WlMessage, wl_client::WlClient};

const STRIDE: usize = 4;

impl WlClient {
    pub fn wl_shm_format(event: &Vec<u8>) {
        let mut offset = 0;
        println!("Received pixel format: {:x}", event.read_u32(&mut offset));
    }

    pub fn wl_shm_create_pool(&mut self, width: usize, height: usize) -> Result<(), Box<dyn Error>> {
        self.current_id += 1;
        self.shm_pool = Some(shm::ShmPool::new(width, height, self.current_id)?);
        let mut grid: Vec<u32> = vec![0xffffffff; width * height];
        for (pos, color) in grid.iter_mut().enumerate() {
            if (pos & 1) == 0 {
                *color = 0xff000000;
            }
        }
        self.shm_pool.as_mut().unwrap().write(&grid, 0);

        let object = self.shm_id.ok_or(UnsetErr("shm_id".to_string()))?;
        const OPCODE: u16 = 0;
        const REQ_SIZE: u16 = 16;
        let id          = self.shm_pool.as_ref().unwrap().id;
        let fds         = [self.shm_pool.as_ref().unwrap().fd];
        let shm_size    = self.shm_pool.as_ref().unwrap().size;

        let mut request = vec![0u8; REQ_SIZE as usize];
        let mut offset: usize = 0;

        // Request header
        request.write_u32(&object,          &mut offset);
        request.write_u16(&OPCODE,          &mut offset);
        request.write_u16(&REQ_SIZE,        &mut offset);

        // Id, size of shm pool
        request.write_u32(&id,                  &mut offset);
        request.write_u32(&(shm_size as u32),   &mut offset);
        
        let mut ancillary_buf = [0u8; 32];
        let mut ancillary = SocketAncillary::new(&mut ancillary_buf[..]);
        
        if !ancillary.add_fds(&fds[..]) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to add FDs to ancillary data",
            ).into());
        }

        self.socket.send_vectored_with_ancillary(&[std::io::IoSlice::new(&request)], &mut ancillary)?;

        Ok(())
    }

    pub fn wl_shm_pool_create_buffer(
        &mut self,
        shm_offset: u32,
        width:      u32,
        height:     u32
    ) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.shm_pool.as_ref().unwrap().id;
        const REQ_SIZE: u16 = 32;
        const OPCODE: u16 = 0;

        let stride: u32 = width * 4;
        self.current_id += 1;
        let id = self.current_id;
        let format = 0;

        let mut offset: usize = 0;
        let mut request = vec![0u8; REQ_SIZE as usize];

        request.write_u32(&object, &mut offset);
        request.write_u16(&OPCODE, &mut offset);
        request.write_u16(&REQ_SIZE, &mut offset);

        request.write_u32(&id,          &mut offset);
        request.write_u32(&shm_offset,  &mut offset);
        request.write_u32(&width,       &mut offset);
        request.write_u32(&height,      &mut offset);
        request.write_u32(&stride,      &mut offset);
        request.write_u32(&format,      &mut offset);

        self.socket.write(&request)?;

        self.buffer_id = Some(self.current_id);

        Ok(())
    }

}
