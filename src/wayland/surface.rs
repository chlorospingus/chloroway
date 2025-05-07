use std::{error::Error, io::Write, sync::atomic::Ordering};

use crate::wayland::{vec_utils::WlMessage, wl_client::WlClient};

use std::fmt;

#[derive(Debug)]
pub struct UnsetErr (pub String);

impl Error for UnsetErr {}
impl fmt::Display for UnsetErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is not set!", self.0)
    }
}

impl WlClient {
    pub fn wl_compositor_create_surface(&mut self) -> Result<(), Box<dyn Error>> {
        let object = self.compositor_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("compositor_id".to_string()).into());
        }
        const OPCODE: u16 = 0;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object, &mut offset);
        request.write_u16(&OPCODE, &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
        request.write_u32(&current_id, &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        self.surface_id.store(current_id, Ordering::Relaxed);

        Ok(())
    }

    pub fn wl_surface_attach(&mut self) -> Result<(), Box<dyn Error>> {
        let object = self.surface_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("surface_id".to_string()).into());
        }
        const OPCODE: u16 = 1;
        const MSG_SIZE: u16 = 20;
        let buffer = self.buffer_id.load(Ordering::Relaxed);
        if buffer == 0 {
            return Err(UnsetErr("buffer_id".to_string()).into());
        }
        const X: u32 = 0;
        const Y: u32 = 0;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);
        request.write_u32(&buffer,   &mut offset);
        request.write_u32(&X,	     &mut offset);
        request.write_u32(&Y,	     &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }

    pub fn wl_surface_commit(&mut self) -> Result<(), Box<dyn Error>> {
        let object = self.surface_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("surface_id".to_string()).into());
        }
        const OPCODE: u16 = 6;
        const MSG_SIZE: u16 = 8;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }

    pub fn xdg_wm_base_pong(&mut self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let object = self.xdg_wm_base_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("xdg_wm_base_id".to_string()).into());
        }
        const OPCODE: u16 = 3;
        const MSG_SIZE: u16 = 12;
        let serial = u32::from_ne_bytes(event[0..4].try_into().unwrap());

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,	 &mut offset);
        request.write_u16(&OPCODE,	 &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);
        request.write_u32(&serial,   &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }
}
