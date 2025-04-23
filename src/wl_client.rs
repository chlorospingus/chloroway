use std::{env::var, error::Error, io::Read, os::unix::net::UnixStream, u32};

use crate::shm;

struct WlHeader {
    object: u32,
    opcode: u16,
    size: u16
}

pub struct WlClient {
    pub socket:         UnixStream,
    pub current_id:     u32,
    pub registry_id:    Option<u32>,
    pub shm_id:         Option<u32>,
    pub shm_pool:       Option<shm::ShmPool>
}

impl WlClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sock = UnixStream::connect(format!(
            "{}/{}",
            var("XDG_RUNTIME_DIR")?,
            var("WAYLAND_DISPLAY")?
        ))?;

        let res = WlClient {
            socket:      sock,
            current_id:  1, 
            registry_id: None,
            shm_id:      None,
            shm_pool:    None,
        };

        Ok(res)
    }

    pub fn read_event(&mut self) -> Result<(), Box<dyn Error>> {
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
}

