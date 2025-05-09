#![feature(unix_socket_ancillary_data)]

use std::{error::Error, io::{IoSliceMut, Write}, os::unix::net::{AncillaryData, SocketAncillary}, sync::atomic::Ordering};

use crate::wayland::{shm, vec_utils::WlMessage, wl_client::WlClient, surface::UnsetErr};


impl WlClient {
    pub fn wl_seat_capabilities(&self, event: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut offset: usize = 0;
        let capability = event.read_u32(&mut offset);
        println!(
            "Received seat capabilities:\n\tPointer: {}\n\tKeyboard: {}\n\tTouch: {}",
            (capability & 1) > 0,
            (capability & 2) > 0,
            (capability & 4) > 0,
        );
        if (capability & 2) > 0 {
            self.wl_seat_get_keyboard()?;
        }

        Ok(())
    }

    pub fn wl_seat_name(&self, event: &Vec<u8>) {
        let mut offset: usize = 0;
        let name = event.read_string(&mut offset);
        println!("Recieved seat name: {}", name);
    }

    pub fn wl_seat_get_keyboard(&self) -> Result<(), Box<dyn Error>> {
        let object = self.seat_id.load(Ordering::Relaxed);
        if object == 0 {
            return Err(UnsetErr("keyboard_id".to_string()).into())
        }
        const OPCODE: u16 = 1;
        const MSG_SIZE: u16 = 12;

        let mut request = vec![0u8; MSG_SIZE as usize];
        let mut offset: usize = 0;
        request.write_u32(&object, &mut offset);
        request.write_u16(&OPCODE, &mut offset);
        request.write_u16(&MSG_SIZE, &mut offset);

        let current_id = self.current_id.fetch_add(1, Ordering::Relaxed) + 1;
        self.keyboard_id.store(current_id, Ordering::Relaxed);
        request.write_u32(&current_id, &mut offset);

        self.socket.lock().unwrap().write(&request)?;
        
        Ok(())
    }

    pub fn wl_keyboard_keymap(&self, event: &Vec<u8>, fd: i32) -> Result<(), Box<dyn Error>>{
        let mut offset: usize = 0;
        let format = event.read_u32(&mut offset);
        let size = event.read_u32(&mut offset);

        *self.keymap.lock().unwrap() = Some(shm::ShmPool::from_fd(fd, size as usize)?);

        Ok(())
    }

    pub fn wl_keyboard_key(&self, event: &Vec<u8>) {
        let mut offset: usize = 0;
        let serial = event.read_u32(&mut offset);
        let time = event.read_u32(&mut offset);
        let key = event.read_u32(&mut offset);
        let state = event.read_u32(&mut offset);
        println!(
            "Received key:\n\tSerial: {}\n\tTime: {}\n\tKey: {}\n\tState: {}",
            serial,
            time,
            key,
            state
        )
    }
}
