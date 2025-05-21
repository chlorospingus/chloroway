use std::{collections::HashMap, env::var, error::Error, fmt::Debug, io::{IoSliceMut, Write}, os::unix::net::{AncillaryData, SocketAncillary, UnixStream}, sync::{atomic::{AtomicBool, AtomicU32, Ordering}, mpsc, Arc, Mutex, RwLock}, thread::{self}, time::Duration, u32};

use crate::{graphics::{circle::Circle, drawable::Drawable, rectangle::Rectangle}, wayland::{shm, surface::UnsetErr, vec_utils::WlMessage, wl_shm::wl_buffer}};

struct WlHeader {
    object: u32,
    opcode: u16,
    size:   u16,
}

pub struct WlClient {
    pub socket:             Mutex<UnixStream>,
    pub running:            AtomicBool,
    pub current_id:         AtomicU32,
    pub registry_id:        AtomicU32,
    pub shm_id:             AtomicU32,
    pub shmpool_id:         AtomicU32,
    pub shm_pool:           Mutex<shm::ShmPool>,
    pub compositor_id:      AtomicU32,
    pub surface_id:         AtomicU32,
    pub active_buffer:      AtomicBool,
    pub buffer1:            Mutex<Option<wl_buffer>>,
    pub buffer2:            Mutex<Option<wl_buffer>>,
    pub frame_hint_id:      AtomicU32,
    pub xdg_wm_base_id:     AtomicU32,
    pub layer_shell_id:     AtomicU32,
    pub layer_surface_id:   AtomicU32,
    pub seat_id:            AtomicU32,
    pub keyboard_id:        AtomicU32,
    pub keymap_fd:          Mutex<Option<shm::ShmPool>>,
    pub keymap:             RwLock<Option<HashMap<u32, Vec<String>>>>,
    pub drawables:          Mutex<Vec<Box<dyn Drawable>>>,
}

impl WlClient {
    pub fn run() -> Result<(), Box<dyn Error>> {
        let sock = UnixStream::connect(format!(
            "{}/{}",
            var("XDG_RUNTIME_DIR")?,
            var("WAYLAND_DISPLAY")?
        ))?;
        sock.set_nonblocking(true)?;

        let mut arc_wl_client = Arc::new(WlClient {
            socket:             Mutex::new(sock),
            running:            AtomicBool::from(false),
            current_id:         AtomicU32::from(1),
            registry_id:        AtomicU32::from(0),
            shm_id:             AtomicU32::from(0),
            shmpool_id:         AtomicU32::from(0),
            shm_pool:           Mutex::new(shm::ShmPool::new(800 * 800 * 4 * 2)?),
            active_buffer:      AtomicBool::from(false),
            buffer1:            Mutex::new(None),
            buffer2:            Mutex::new(None),
            frame_hint_id:      AtomicU32::from(0),
            compositor_id:      AtomicU32::from(0),
            surface_id:         AtomicU32::from(0),
            xdg_wm_base_id:     AtomicU32::from(0),
            layer_shell_id:     AtomicU32::from(0),
            layer_surface_id:   AtomicU32::from(0),
            seat_id:            AtomicU32::from(0),
            keyboard_id:        AtomicU32::from(0),
            keymap:             RwLock::new(None),
            keymap_fd:          Mutex::new(None),
            drawables:          Mutex::new(Vec::new()),
        }); 
        arc_wl_client.wl_display_get_registry();
        if let Ok(mut drawables) = arc_wl_client.drawables.lock() {
            drawables.push(Rectangle::new(50, 50, 300, 300, 16, 0xffff8800).into());
            drawables.push(Circle::new(150, 80, 25, 0xff00ffff).into());
        }
        arc_wl_client.running.store(true, Ordering::Relaxed);

        let wl_client = arc_wl_client.clone();
        let readloop = thread::Builder::new().name("readloop".to_string()).spawn(move || {
            while wl_client.running.load(Ordering::Relaxed) {
                wl_client.read_event();
            }
        })?;

        readloop.join();

        Ok(())
    }

    pub fn read_event(self: &Arc<Self>) -> Result<(), Box<dyn Error + '_>> {
        // TODO: Don't realloc header and event

        // FIXME: Using fd like this is unreliable because fd could be before or after
        // event it was intended to be with
        let mut fd = 0;

        let mut header = vec![0u8; 8];
        let socket = self.socket.lock().unwrap();
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
                scm_rights.for_each(|received| {
                    fd = received;
                });
            }
        }

        drop(socket);

        if header.object == self.registry_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_registry::global
            self.wl_registry_global(&event)?;
        }
        else if header.object == 1 && header.opcode == 0 { // wl_display::error
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
            // println!("Preferred buffer scale: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
        }
        else if header.object == self.surface_id.load(Ordering::Relaxed) && header.opcode == 3 { // wl_surface::preferred_buffer_transform
            // println!("Preferred buffer transform: {}", i32::from_ne_bytes(event[0..4].try_into().unwrap()));
        }
        else if header.object == self.seat_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_seat::capabilities
            self.wl_seat_capabilities(&event)?;
        }
        else if header.object == self.seat_id.load(Ordering::Relaxed) && header.opcode == 1 { // wl_seat::name
            self.wl_seat_name(&event);
        }
        else if header.object == self.keyboard_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_keyboard::keymap
            self.wl_keyboard_keymap(&event, fd)?;
        }
        else if header.object == self.keyboard_id.load(Ordering::Relaxed) && header.opcode == 3 { // wl_keyboard::key
            let wl_client = self.clone();
            thread::spawn(move || {
                wl_client.wl_keyboard_key(&event);
            });
        }
        else if header.object == self.frame_hint_id.load(Ordering::Relaxed) && header.opcode == 0 { // wl_callback<frame_hint>::done
            self.wl_surface_frame()?;
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

    pub fn destroy_object(&self, id: u32, opcode: u16) {
        if id == 0 {
            return;
        }
        const REQ_SIZE: u16 = 8;

        let mut request = vec![0; REQ_SIZE as usize];
        let mut offset = 0;

        request.write_u32(&id,       &mut offset);
        request.write_u16(&opcode,   &mut offset);
        request.write_u16(&REQ_SIZE, &mut offset);

        self.socket.lock().unwrap().write(&request);
        self.shmpool_id.store(0, Ordering::Relaxed);
    }

    pub fn exit(&self) -> Result<(), Box<dyn Error + '_>> {
        println!("Exiting!");
        self.destroy_object(self.layer_surface_id.load(Ordering::Relaxed), 7);
        self.destroy_object(self.buffer1.lock()?.as_ref().ok_or(UnsetErr("buffer1".to_string()))?.id, 0);
        self.keymap_fd.lock().unwrap().take();
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Debug for WlClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WlClient {{
    current_id: {},
    registry_id: {},
    shm_id: {},
    shmpool_id: {},
    buffer1: {:?},
    buffer2: {:?},
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
    self.shmpool_id.load(Ordering::Relaxed),
    self.buffer1.lock().unwrap(),
    self.buffer2.lock().unwrap(),
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
