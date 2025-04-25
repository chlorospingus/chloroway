use std::{error::Error, io::Write};

use crate::{vec_utils::WlMessage, WlClient};

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
        if self.compositor_id.is_none() {
            return Err(UnsetErr("compositor_id".to_string()).into());
        }

        let object = self.compositor_id.unwrap();
        const OPCODE: u16 = 0;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object, &mut offset);
        request.write_u16(&OPCODE, &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        self.current_id += 1;
        request.write_u32(&self.current_id, &mut offset);

        self.socket.write(&request)?;

        self.surface_id = Some(self.current_id);

        Ok(())
    }

    pub fn wl_surface_attach(&mut self) -> Result<(), Box<dyn Error>> {
        if self.surface_id.is_none() {
            return Err(Box::new(UnsetErr("surface_id".to_string())));
        }
        let object = self.surface_id.unwrap();
        const OPCODE: u16 = 1;
        const MSG_SIZE: u16 = 20;
        const X: u32 = 0;
        const Y: u32 = 0;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);
        request.write_u32(&X,	     &mut offset);
        request.write_u32(&Y,	     &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }

    pub fn xdg_wm_base_pong(&mut self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        if self.xdg_wm_base_id.is_none() {
            return Err(Box::new(UnsetErr("xdg_wm_base_id".to_string())));
        }
        let object = self.xdg_wm_base_id.unwrap();
        const OPCODE: u16 = 3;
        const MSG_SIZE: u16 = 12;
        let serial = u32::from_ne_bytes(event[0..4].try_into().unwrap());

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,	 &mut offset);
        request.write_u16(&OPCODE,	 &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);
        request.write_u32(&serial,   &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }
}
