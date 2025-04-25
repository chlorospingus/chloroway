use std::{error::Error, io::Write};
use crate::{surface::UnsetErr, vec_utils::WlMessage, WlClient};

const NAMESPACE: &str = "chlorostart";
const OVERLAY: u32 = 3;

impl WlClient {
    pub fn layer_shell_get_layer_surface(&mut self) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.layer_shell_id.unwrap();
        const OPCODE: u16 = 0;
        let msg_size: u16 = 28 + (NAMESPACE.len()+1).next_multiple_of(4) as u16;
        let mut request = vec![0u8; msg_size as usize];
        let mut offset: usize = 0;
        let output: u32 = 0;

        request.write_u32(&object,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&msg_size, &mut offset);

        self.current_id += 1;
        request.write_u32(&self.current_id, &mut offset);
        request.write_u32(&self.surface_id.unwrap(), &mut offset);
        request.write_u32(&output, &mut offset);
        request.write_u32(&OVERLAY, &mut offset);
        request.write_string(&NAMESPACE.to_string(), &mut offset);

        self.socket.write(&request)?;

        Ok(())
    }
}
