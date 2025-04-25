use std::{error::Error, io::Write};
use crate::{surface::UnsetErr, vec_utils::WlMessage, WlClient};

const NAMESPACE: &str = "chlorostart";
const OVERLAY: u32 = 3;

impl WlClient {
    pub fn layer_shell_get_layer_surface(&mut self) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_shell_id.unwrap();
        const OPCODE: u16 = 0;
        let msg_size: u16 = 28 + (NAMESPACE.len()+1).next_multiple_of(4) as u16;
        let output: u32 = 0;

        let mut request = vec![0u8; msg_size as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&msg_size, &mut offset);

        self.current_id += 1;
        request.write_u32(&self.current_id, &mut offset);
        self.layer_surface_id = Some(self.current_id);
        request.write_u32(&self.surface_id.unwrap(), &mut offset);
        request.write_u32(&output, &mut offset);
        request.write_u32(&OVERLAY, &mut offset);
        request.write_string(&NAMESPACE.to_string(), &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }

    pub fn layer_surface_configure(&mut self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut offset: usize = 0;
        let serial = event.read_u32(&mut offset);
        let width  = event.read_u32(&mut offset);
        let height = event.read_u32(&mut offset);

        // TODO: Resize based on configure

        // Ack configure
        let object = self.layer_surface_id.unwrap();
        const OPCODE: u16 = 6;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        offset = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&serial, &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }

    pub fn layer_surface_set_size(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_surface_id.unwrap();
        const OPCODE: u16 = 0;
        const MSG_SIZE: u16 = 20;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&width,    &mut offset);
        request.write_u32(&height,   &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }

    pub fn layer_surface_set_keyboard_interactivity(&mut self) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_surface_id.unwrap();
        const OPCODE: u16 = 4;
        const MSG_SIZE: u16 = 12;
        const EXCLUSIVE: u32 = 1; // exclusize keyboard focus

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        request.write_u32(&EXCLUSIVE, &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }
}
