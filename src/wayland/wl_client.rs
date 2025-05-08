use std::{env::var, error::Error, fmt::Debug, io::Read, os::unix::net::UnixStream, sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, u32};

use crate::wayland::shm;

struct WlHeader {
    object: u32,
    opcode: u16,
    size:   u16,
}

pub struct WlClient {
    pub socket:             Mutex<UnixStream>,
    pub current_id:         AtomicU32,
    pub registry_id:        AtomicU32,
    pub shm_id:             AtomicU32,
    pub shm_pool:           Mutex<Option<shm::ShmPool>>,
    pub buffer_id:          AtomicU32,
    pub compositor_id:      AtomicU32,
    pub surface_id:         AtomicU32,
    pub xdg_wm_base_id:     AtomicU32,
    pub layer_shell_id:     AtomicU32,
    pub layer_surface_id:   AtomicU32,
}

impl WlClient {
    pub fn run() -> Result<Arc<Self>, Box<dyn Error>> {
        let sock = UnixStream::connect(format!(
            "{}/{}",
            var("XDG_RUNTIME_DIR")?,
            var("WAYLAND_DISPLAY")?
        ))?;

        let mut wl_client = Arc::new(WlClient {
            socket:             Mutex::new(sock),
            current_id:         AtomicU32::from(1),
            registry_id:        AtomicU32::from(0),
            shm_id:             AtomicU32::from(0),
            shm_pool:           Mutex::new(None),
            buffer_id:          AtomicU32::from(0),
            compositor_id:      AtomicU32::from(0),
            surface_id:         AtomicU32::from(0),
            xdg_wm_base_id:     AtomicU32::from(0),
            layer_shell_id:     AtomicU32::from(0),
            layer_surface_id:   AtomicU32::from(0),
        });

        let mut wl_client2 = wl_client.clone();

        wl_client.wl_display_get_registry();
        let readloop = thread::spawn(move || {
            loop {
                wl_client2.read_event();
            }
        });

        readloop.join();

        Ok(wl_client)
    }

    pub fn read_event(&self) -> Result<(), Box<dyn Error>> {
        // TODO: Don't realloc header and event

        let mut header = vec![0u8; 8];
        let mut socket = self.socket.lock().unwrap();
        socket.read_exact(&mut header)?;

        let header = WlHeader {
            object: u32::from_ne_bytes(header[0..4].try_into()?),
            opcode: u16::from_ne_bytes(header[4..6].try_into()?),
            size:   u16::from_ne_bytes(header[6..8].try_into()?)
        };

        let mut event = vec![0u8; header.size as usize - 8];
        socket.read_exact(&mut event)?;
        drop(socket);

        println!(
            "Received event:\n\tObject: {}\n\tOpcode: {}\n\tSize: {}",
            header.object,
            header.opcode,
            header.size
        );
        if header.object == self.registry_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_registry::global
            self.wl_registry_global(&event)?;
        }
        else if header.object == self.registry_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_display::error
            WlClient::wl_display_error(&event);
        }
        else if header.object == self.shm_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_shm::format
            WlClient::wl_shm_format(&event);
        }
        else if header.object == self.xdg_wm_base_id.load(Ordering::Relaxed) && header.opcode == 0 { // xdg_wm_base::ping
            self.xdg_wm_base_pong(&event)?;
        }
        else if header.object == self.layer_surface_id.load(Ordering::Relaxed) && header.opcode == 0 { // zwlr_layer_surface::configure
            self.layer_surface_configure(&event)?;
        }
        else if header.object == self.surface_id.load(Ordering::Relaxed) && header.opcode == 2 { // wl_surface::preferred_buffer_scale
            println!("Preferred buffer scale: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
        }
        else if header.object == self.surface_id.load(Ordering::Relaxed) && header.opcode == 3 { // wl_surface::preferred_buffer_transform
            println!("Preferred buffer transform: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
        }

        Ok(())
    }
}

