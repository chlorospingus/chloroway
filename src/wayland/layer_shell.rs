use std::{error::Error, io::Write, sync::atomic::Ordering};
use crate::wayland::{surface::UnsetErr, vec_utils::WlMessage, wl_client::WlClient};

const NAMESPACE: &str = "chlorostart";
const OVERLAY: u32 = 3;
const EXCLUSIVE: u32 = 0; // exclusize keyboard focus

impl WlClient {
    pub fn layer_shell_get_layer_surface(&self) -> Result<(), Box<dyn Error>> {
        // TODO: Make sure layer_surface_id isn't already set
        let object: u32 = self.layer_shell_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("layer_shell_id".to_string()).into());
        }
        const OPCODE: u16 = 0;
        let msg_size: u16 = 28 + (NAMESPACE.len()+1).next_multiple_of(4) as u16;
        let output: u32 = 0;

        let mut request = vec![0u8; msg_size as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&msg_size, &mut offset);

        let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
        request.write_u32(&current_id, &mut offset);

        let surface_id = self.surface_id.load(Ordering::Relaxed);
        if surface_id == 0 {
            return Err(UnsetErr("surface_id".to_string()).into());
        }

        request.write_u32(&surface_id, &mut offset);
        request.write_u32(&output, &mut offset);
        request.write_u32(&OVERLAY, &mut offset);
        request.write_string(&NAMESPACE.to_string(), &mut offset);

        self.socket.lock().unwrap().write(&request)?;
        self.layer_surface_id.store(current_id, Ordering::Relaxed);

        Ok(())
    }

    pub fn layer_surface_configure(&self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut offset: usize = 0;
        let serial = event.read_u32(&mut offset);
        let width  = event.read_u32(&mut offset);
        let height = event.read_u32(&mut offset);

        // TODO: Resize based on configure

        // Ack configure
        let object = self.layer_surface_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("layer_surface_id".to_string()).into());
        }
        const OPCODE: u16 = 6;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        offset = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&serial, &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        self.wl_surface_attach()?;
        self.wl_surface_commit()?;

        Ok(())
    }

    pub fn layer_surface_set_size(&self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_surface_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("layer_surface_id".to_string()).into());
        }
        const OPCODE: u16 = 0;
        const MSG_SIZE: u16 = 20;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&width,    &mut offset);
        request.write_u32(&height,   &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }

    pub fn layer_surface_set_keyboard_interactivity(&self) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_surface_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("layer_surface_id".to_string()).into());
        }
        const OPCODE: u16 = 4;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&EXCLUSIVE, &mut offset);

        self.socket.lock().unwrap().write(&request)?;

        Ok(())
    }
}
