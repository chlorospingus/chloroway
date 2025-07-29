use std::sync::atomic::AtomicU32;
use std::{error::Error, io::Write, os::unix::net::SocketAncillary, sync::atomic::Ordering, u8};
use crate::wayland::{surface::UnsetErr, vec_utils::WlMessage, wl_client::WlClient};

const STRIDE: usize = 4;

#[derive(Debug)]
pub struct wl_buffer {
    pub id:         u32,
    pub offset:     usize,
    pub width:      usize,
    pub height:     usize,
    pub ready:      bool,
}

impl WlClient {
    pub fn wl_shm_format(event: &Vec<u8>) {
        // let mut offset = 0;
        // println!("Received pixel format: {:x}", event.read_u32(&mut offset));
    }

    pub fn wl_shm_create_pool(&self) -> Result<(), Box<dyn Error>> {
        let shm_pool = self.shm_pool.lock().unwrap();
        let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
        self.shmpool_id.store(current_id, Ordering::Relaxed);

        let object = self.shm_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("shm_id".to_string()).into());
        }
        const OPCODE: u16 = 0;
        const REQ_SIZE: u16 = 16;
        let id          = self.shmpool_id.load(Ordering::Relaxed);
        let fds         = [shm_pool.fd];
        let shm_size    = shm_pool.size;

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

        self.socket.lock().unwrap().send_vectored_with_ancillary(&[std::io::IoSlice::new(&request)], &mut ancillary)?;

        Ok(())
    }

    pub fn wl_shm_pool_create_buffer(
        &self,
        buffer: &wl_buffer
    ) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.shmpool_id.load(Ordering::Relaxed);
        const REQ_SIZE: u16 = 32;
        const OPCODE: u16 = 0;

        let byte_offset: u32 = buffer.offset as u32 * 4;
        let width: u32 = buffer.width as u32;
        let height: u32 = buffer.height as u32;
        let stride: u32 = width * 4;
        let id = buffer.id;
        let format = 0;

        let mut offset: usize = 0;
        let mut request = vec![0u8; REQ_SIZE as usize];

        request.write_u32(&object, &mut offset);
        request.write_u16(&OPCODE, &mut offset);
        request.write_u16(&REQ_SIZE, &mut offset);

        request.write_u32(&id,          &mut offset);
        request.write_u32(&byte_offset, &mut offset);
        request.write_u32(&width,       &mut offset);
        request.write_u32(&height,      &mut offset);
        request.write_u32(&stride,      &mut offset);
        request.write_u32(&format,      &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }
}
