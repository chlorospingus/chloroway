use std::{io::Write, error::Error, os::unix::net::SocketAncillary};
use crate::{WlClient, vec_utils::WlMessage, shm};

impl WlClient {
    pub fn wl_shm_format(event: &Vec<u8>) {
        let mut offset = 0;
        println!("Received pixel format: {:x}", event.read_u32(&mut offset));
    }

    pub fn wl_shm_create_pool(&mut self) -> Result<(), String> {
        self.current_id += 1;
        self.shm_pool = Some(match shm::ShmPool::new(4096, self.current_id) {
            Ok(val) => val,
            Err(err) => {
                return Err(err.to_string());
            }
        });

        let object = match self.shm_id {
            Some(val) => val,
            None => {
                return Err("error in wl_shm_create_pool: shm_id not set!".to_string());
            }
        };
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
            return Err("Error in wl_shm_create_pool: ancillary.add_fds failed".to_string());
        }

        match self.socket.send_vectored_with_ancillary(&[std::io::IoSlice::new(&request)], &mut ancillary) {
            Ok(bytes) => {
                assert!(bytes == REQ_SIZE as usize);
            }
            Err(err) => {
                return Err(err.to_string());
            }
        };

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

        Ok(())
    }

}
