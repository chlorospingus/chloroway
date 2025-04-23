#![feature(unix_socket_ancillary_data)]
use std::{env, error::Error, io::{Read, Write}, os::unix::net::{UnixStream, SocketAncillary}, u32};

mod shm;
mod wl_shm;
mod vec_utils;
pub use vec_utils::WlMessage;

struct WlHeader {
    object: u32,
    opcode: u16,
    size: u16
}

struct WlClient {
    socket:         UnixStream,
    current_id:     u32,
    registry_id:    Option<u32>,
    shm_id:         Option<u32>,
    shm_pool:       Option<shm::ShmPool>
}

impl WlClient {
    fn new() -> Result<Self, Box<dyn Error>> {
        let sock = WlClient::connect()?;

        let res = WlClient {
            socket:      sock,
            current_id:  1, 
            registry_id: None,
            shm_id:      None,
            shm_pool:    None,
        };

        Ok(res)
    }

    fn read_event(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: Don't realloc header and event

        let mut header = vec![0u8; 8];
        self.socket.read_exact(&mut header)?;

        let header = WlHeader {
            object: u32::from_ne_bytes(header[0..4].try_into()?),
            opcode: u16::from_ne_bytes(header[4..6].try_into()?),
            size:   u16::from_ne_bytes(header[6..8].try_into()?)
        };

        let mut event = vec![0u8; header.size as usize - 8];
        self.socket.read_exact(&mut event)?;

        if header.object == self.registry_id.unwrap() && header.opcode == 0 { // wl_registry::global
            self.wl_registry_global(&event)?;
        }
        else if header.object == 1 && header.opcode == 0 { // wl_display::error
            WlClient::wl_display_error(&event);
        }
        else if self.shm_id.is_some() && header.object == self.shm_id.unwrap() && header.opcode == 0 { // wl_shm::format
            WlClient::wl_shm_format(&event);
        }
        else {
            println!(
                "Received event:\n\tObject: {}\n\tOpcode: {}\n\tSize: {}",
                header.object,
                header.opcode,
                header.size
            );
        }

        Ok(())
    }

    fn connect() -> Result<UnixStream, Box<dyn Error>> {
        let wl_sock_path: String = format!(
            "{}/{}",
            env::var("XDG_RUNTIME_DIR")?,
            env::var("WAYLAND_DISPLAY")?
        );
        let sock = UnixStream::connect(wl_sock_path)?;

        Ok(sock)
    }

    fn wl_display_error(event: &Vec<u8>) {
        let mut offset: usize = 0;
        eprintln!(
            "Received error:\n\tObject: {}\n\tCode: {}\n\tMessage: {}",
            event.read_u32(&mut offset),
            event.read_u32(&mut offset),
            event.read_string(&mut offset)
        );
    }

    fn wl_display_get_registry(&mut self) -> Result<(), Box<dyn Error>> {
        const OBJECT: u32 = 1;
        const OPCODE: u16 = 1;
        const MSG_SIZE: u16 = 12;

        let mut request = [0u8; MSG_SIZE as usize];
        request[0..4].copy_from_slice(&OBJECT.to_ne_bytes());
        request[4..6].copy_from_slice(&OPCODE.to_ne_bytes());
        request[6..8].copy_from_slice(&MSG_SIZE.to_ne_bytes());

        self.current_id += 1;
        request[8..12].copy_from_slice(&self.current_id.to_ne_bytes());
        self.registry_id = Some(self.current_id);

        let written = self.socket.write(&request)?;
        assert!(written == MSG_SIZE.into());

        Ok(())
    }

    fn wl_registry_bind(
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

    fn wl_registry_global(&mut self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
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

        if interface == "wl_shm" {
            self.current_id += 1;
            self.wl_registry_bind(
                &name,
                &interface,
                &version,
                &self.current_id.clone()
            )?;
            self.shm_id = Some(self.current_id);

            self.wl_shm_create_pool()?;
            self.wl_shm_pool_create_buffer(0, 200, 200)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut wl_client = WlClient::new()?;

    wl_client.wl_display_get_registry()?;

    loop {
        wl_client.read_event()?;

        if false {
            break
        }
    }

    Ok(())
}
