use std::{env::var, error::Error, fmt::Debug, io::Read, os::unix::net::UnixStream, u32};

use crate::{shm, vec_utils::WlMessage};

struct WlHeader {
    object: u32,
    opcode: u16,
    size:   u16,
}

#[derive(Clone)]
pub struct Color {
    pub alpha:  u8,
    pub red:    u8,
    pub green:  u8,
    pub blue:   u8,
}

impl Color {
    pub const WHITE: Self = Self {
        alpha: u8::MAX,
        red: u8::MAX,
        green: u8::MAX,
        blue: u8::MAX,
    };

    pub const RED: Self = Self {
        alpha: 0xff,
        red: 0xff,
        green: 0,
        blue: 0,
    };
}

pub struct WlClient {
    pub socket:             UnixStream,
    pub current_id:         u32,
    pub registry_id:        Option<u32>,
    pub shm_id:             Option<u32>,
    pub shm_pool:           Option<shm::ShmPool>,
    pub buffer_id:          Option<u32>,
    pub compositor_id:      Option<u32>,
    pub surface_id:         Option<u32>,
    pub xdg_wm_base_id:     Option<u32>,
    pub layer_shell_id:     Option<u32>,
    pub layer_surface_id:   Option<u32>,
}

impl WlClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sock = UnixStream::connect(format!(
            "{}/{}",
            var("XDG_RUNTIME_DIR")?,
            var("WAYLAND_DISPLAY")?
        ))?;

        let res = WlClient {
            socket:             sock,
            current_id:         1, 
            registry_id:        None,
            shm_id:             None,
            shm_pool:           None,
            buffer_id:          None,
            compositor_id:      None,
            surface_id:         None,
            xdg_wm_base_id:     None,
            layer_shell_id:     None,
            layer_surface_id:   None,
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
        else if self.xdg_wm_base_id.is_some() && header.object == self.xdg_wm_base_id.unwrap() && header.opcode == 0 { // xdg_wm_base::ping
            self.xdg_wm_base_pong(&event)?;
        }
        else if Some(header.object) == self.layer_surface_id && header.opcode == 0 { // zwlr_layer_surface::configure
            self.layer_surface_configure(&event)?;
        }
        else if Some(header.object) == self.surface_id && header.opcode == 2 { // wl_surface::preferred_buffer_scale
            println!("Preferred buffer scale: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
        }
        else if Some(header.object) == self.surface_id && header.opcode == 3 { // wl_surface::preferred_buffer_transform
            println!("Preferred buffer transform: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
            dbg!(self);
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

impl Debug for WlClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
"WlClient {{
    current_id: {},
    registry_id: {},
    shm_id: {},
    buffer_id: {},
    compositor_id: {},
    surface_id: {},
    xdg_wm_base_id: {},
    layer_shell_id: {},
    layer_surface_id: {},
}}",
            self.current_id,
            self.registry_id.unwrap_or(0),
            self.shm_id.unwrap_or(0),
            self.buffer_id.unwrap_or(0),
            self.compositor_id.unwrap_or(0),
            self.surface_id.unwrap_or(0),
            self.xdg_wm_base_id.unwrap_or(0),
            self.layer_shell_id.unwrap_or(0),
            self.layer_surface_id.unwrap_or(0),
        )    
    }
}
