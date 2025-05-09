#![feature(unix_socket_ancillary_data)]

use std::{env::var, error::Error, fmt::Debug, io::{IoSliceMut, Read}, os::unix::net::{AncillaryData, SocketAncillary, UnixStream}, sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, u32};

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
    pub shmpool_id:         AtomicU32,
    pub shm_pool:           Mutex<Option<shm::ShmPool>>,
    pub buffer_id:          AtomicU32,
    pub compositor_id:      AtomicU32,
    pub surface_id:         AtomicU32,
    pub xdg_wm_base_id:     AtomicU32,
    pub layer_shell_id:     AtomicU32,
    pub layer_surface_id:   AtomicU32,
    pub seat_id:            AtomicU32,
    pub keyboard_id:        AtomicU32,
    pub keymap:             Mutex<Option<shm::ShmPool>>,
}

impl WlClient {
    pub fn run() -> Result<(), Box<dyn Error>> {
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
            shmpool_id:         AtomicU32::from(0),
            shm_pool:           Mutex::new(None),
            buffer_id:          AtomicU32::from(0),
            compositor_id:      AtomicU32::from(0),
            surface_id:         AtomicU32::from(0),
            xdg_wm_base_id:     AtomicU32::from(0),
            layer_shell_id:     AtomicU32::from(0),
            layer_surface_id:   AtomicU32::from(0),
            seat_id:            AtomicU32::from(0),
            keyboard_id:        AtomicU32::from(0),
            keymap:             Mutex::new(None),
        });

        let mut wl_client2 = wl_client.clone();

        wl_client.wl_display_get_registry();
        let readloop = thread::spawn(move || {
            loop {
                wl_client2.read_event();
            }
        });

        readloop.join();

        Ok(())
    }

    pub fn read_event(&self) -> Result<(), Box<dyn Error>> {
        // TODO: Don't realloc header and event

        // FIXME: Using fd like this is unreliable because fd could be before or after
        // event it was intended to be with
        let mut fd = 0;

        let mut header = vec![0u8; 8];
        let mut socket = self.socket.lock().unwrap();
        let mut ancillary_buf = [0; 128];
        let mut ancillary = SocketAncillary::new(&mut ancillary_buf);

        socket.recv_vectored_with_ancillary(
            &mut [IoSliceMut::new(header.as_mut_slice())],
            &mut ancillary
        )?;
        for ancillary_result in ancillary.messages() {
            if let AncillaryData::ScmRights(scm_rights) = ancillary_result.unwrap() {
                scm_rights.for_each(|received| {
                    fd = received;
                });
            }
        }

        let header = WlHeader {
            object: u32::from_ne_bytes(header[0..4].try_into()?),
            opcode: u16::from_ne_bytes(header[4..6].try_into()?),
            size:   u16::from_ne_bytes(header[6..8].try_into()?)
        };

        let mut event = vec![0u8; header.size as usize - 8];
        socket.recv_vectored_with_ancillary(
            &mut [IoSliceMut::new(event.as_mut_slice())],
            &mut ancillary
        )?;
        for ancillary_result in ancillary.messages() {
            if let AncillaryData::ScmRights(scm_rights) = ancillary_result.unwrap() {
                scm_rights.for_each(|fd| {
                    println!("found {}", fd);
                });
            }
        }

        drop(socket);

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
        else if header.object == self.seat_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_seat::capabilities
            self.wl_seat_capabilities(&event)?;
        }
        else if header.object == self.seat_id.load(Ordering::Relaxed) && header.opcode == 1 { // wl_seat::name
            self.wl_seat_name(&event);
        }
        else if header.object == self.keyboard_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_keyboard::keymap
            self.wl_keyboard_keymap(&event, fd);
        }
        else if header.object == self.keyboard_id.load(Ordering::Relaxed) && header.opcode == 3 { // wl_keyboard::key
            self.wl_keyboard_key(&event);
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
        write!(f, "WlClient {{
    current_id: {},
    registry_id: {},
    shm_id: {},
    buffer_id: {},
    compositor_id: {},
    surface_id: {},
    xdg_wm_base_id: {},
    layer_shell_id: {},
    layer_surface_id: {},
    seat_id: {},
    keyboard_id: {},
}}",
            self.current_id.load(Ordering::Relaxed),
            self.registry_id.load(Ordering::Relaxed),
            self.shm_id.load(Ordering::Relaxed),
            self.buffer_id.load(Ordering::Relaxed),
            self.compositor_id.load(Ordering::Relaxed),
            self.surface_id.load(Ordering::Relaxed),
            self.xdg_wm_base_id.load(Ordering::Relaxed),
            self.layer_shell_id.load(Ordering::Relaxed),
            self.layer_surface_id.load(Ordering::Relaxed),
            self.seat_id.load(Ordering::Relaxed),
            self.keyboard_id.load(Ordering::Relaxed),
        )
    }
}
