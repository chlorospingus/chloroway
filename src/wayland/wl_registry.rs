use crate::wayland::{surface::UnsetErr, vec_utils::WlMessage, wl_client::WlClient};
use std::{error::Error, io::Write, sync::atomic::{AtomicU32, Ordering}};

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

        self.layer_surface_set_size(800, 800)?;
        self.layer_surface_set_keyboard_interactivity()?;
        self.wl_surface_commit()?;

        self.wl_shm_create_pool(800, 800)?;
        self.wl_shm_pool_create_buffer(0, 800, 800)?;

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

        let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
        request.write_u32(&current_id, &mut offset);

        self.socket.write(&request)?;
        self.registry_id = AtomicU32::from(current_id);

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
            let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &current_id
            )?;
            self.shm_id = Some(current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "wl_compositor" {
            let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &current_id
            )?;
            self.compositor_id = Some(current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "xdg_wm_base" {
            let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &current_id
            )?;
            self.xdg_wm_base_id = Some(current_id);
            self.init_toplevel().unwrap_or_else(|err| {eprintln!("{}", err)});
        }

        if interface == "zwlr_layer_shell_v1" {
            let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &current_id
            )?;
            self.layer_shell_id = Some(current_id);
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
    ) -> Result<(), Box<dyn Error>> {
        let object: u32 = self.registry_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("registry_id".to_string()).into());
        }
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

        self.socket.write(&request)?;

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
