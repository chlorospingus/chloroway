use crate::{surface::UnsetErr, vec_utils::WlMessage, WlClient};
use std::{io::Write, error::Error};

impl WlClient {
    fn init_toplevel(&mut self) -> Result<(), Box<dyn Error>> {
        if self.shm_id.is_none() {
            return Err(Box::new(UnsetErr("shm_id".to_string())));
        }
        if self.compositor_id.is_none() {
            return Err(Box::new(UnsetErr("compositor_id".to_string())));
        }
        if self.xdg_wm_base_id.is_none() {
            return Err(Box::new(UnsetErr("xdg_wm_base_id".to_string())));
        }
        if self.layer_shell_id.is_none() {
            return Err(UnsetErr("layer_shell_id".to_string()).into());
        }
        println!("Initializing toplevel!");
        self.wl_compositor_create_surface()?;
        self.layer_shell_get_layer_surface()?;

        self.layer_surface_set_size(200, 200)?;
        self.layer_surface_set_keyboard_interactivity()?;
        self.wl_surface_commit()?;

        self.wl_shm_create_pool()?;
        self.wl_shm_pool_create_buffer(0, 200, 200)?;
        self.wl_surface_attach()?;
        // self.wl_surface_commit()?;

        Ok(())
    }

    pub fn wl_display_get_registry(&mut self) -> Result<(), Box<dyn Error>> {
        const OBJECT: u32 = 1;
        const OPCODE: u16 = 1;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;

        request.write_u32(&OBJECT,   &mut offset);
        request.write_u16(&OPCODE,   &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        self.current_id += 1;
        request.write_u32(&self.current_id, &mut offset);
        self.registry_id = Some(self.current_id);

        let written = self.socket.write(&request)?;
        assert!(written == MSG_SIZE.into());

        Ok(())
    }

    pub fn wl_registry_global(&mut self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut offset: usize = 0;

        let name        = event.read_u32(&mut offset);
        let interface   = event.read_string(&mut offset);
        let version     = event.read_u32(&mut offset);

        println!(
            "Received global:\n\tName: {}\n\tInterface: {}\n\tVersion: {}",
            name,
            interface,
            version,
        );

        // TODO: Collapse these into one line (probably using a macro)

        if interface == "wl_shm" {
            self.current_id += 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &self.current_id.clone()
            )?;
            self.shm_id = Some(self.current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "wl_compositor" {
            self.current_id += 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &self.current_id.clone()
            )?;
            self.compositor_id = Some(self.current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "xdg_wm_base" {
            self.current_id += 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &self.current_id.clone()
            )?;
            self.xdg_wm_base_id = Some(self.current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "zwlr_layer_shell_v1" {
            self.current_id += 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &self.current_id.clone()
            )?;
            self.layer_shell_id = Some(self.current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        Ok(())
    }

    pub fn wl_registry_bind(
        &mut self,
        name: &u32,
        interface: &String,
        version: &u32,
        id: &u32
    ) -> Result<(), String> {
        let object: u32 = match self.registry_id {
            Some(id) => id,
            None => return Err(String::from("wl_registry_bind failed: wl_state.registry_id not set!"))
        };
        const OPCODE: u16 = 0;

        let req_size: u16 = 24 + ((interface.len() as u16+3) & (u16::MAX-3));
        let mut request = vec![0u8; req_size as usize];
        let mut offset: usize = 0;

        request.write_u32    (&object,    &mut offset);
        request.write_u16    (&OPCODE,    &mut offset);
        request.write_u16    (&req_size,  &mut offset);

        request.write_u32    (&name,      &mut offset);
        request.write_string (&interface, &mut offset);
        request.write_u32    (&version,   &mut offset);
        request.write_u32    (&id,        &mut offset);

        match self.socket.write(&request) {
            Ok(bytes) => {
                assert!(bytes == req_size as usize)
            }
            Err(err) => {
                return Err(err.to_string());
            }
        };

        Ok(())
    }

    pub fn wl_display_error(event: &Vec<u8>) {
        let mut offset: usize = 0;
        eprintln!(
            "Received error:\n\tObject: {}\n\tCode: {}\n\tMessage: {}",
            event.read_u32(&mut offset),
            event.read_u32(&mut offset),
            event.read_string(&mut offset)
        );
    }
}
