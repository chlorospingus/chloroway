#![feature(unix_socket_ancillary_data)]
use std::{env, error::Error, io::{Read, Write}, os::unix::net::{UnixStream, SocketAncillary, AncillaryData}, u32};

use shm::shm::ShmPool;
mod shm;

struct WlState {
    socket:         UnixStream,
    current_id:     u32,
    registry_id:    Option<u32>,
    shm_id:         Option<u32>,
    shm_pool:       Option<ShmPool>
}

struct WlHeader {
    object: u32,
    opcode: u16,
    size: u16
}

trait WlMessage {
    /// Write a u32 to self at offset and increment offset by four
    fn write_u32(&mut self, value: &u32, offset: &mut usize);
    /// Write a u16 to self at offset and increment offset by four
    fn write_u16(&mut self, value: &u16, offset: &mut usize);
    /// Write a string to self at offset
    /// and increment offset by string length rounded up to four bytes
    fn write_string(&mut self, str: &String, offset: &mut usize);
    /// Read a u32 from self at offset and increment offset by four
    fn read_u32(&self, offset: &mut usize) -> u32;
    /// Read a u16 from self at offset and increment offset by two
    fn read_u16(&self, offset: &mut usize) -> u16;
    /// Read a string from self at offset 
    /// and increment offset by string length rounded up to four bytes
    fn read_string(&self, offset: &mut usize) -> String;
}

impl WlMessage for Vec<u8> {
    fn write_u32(&mut self, value: &u32, offset: &mut usize) {
        self[*offset..*offset+4].copy_from_slice(&value.to_ne_bytes());
        *offset += 4;
    }

    fn write_u16(&mut self, value: &u16, offset: &mut usize) {
        self[*offset..*offset+2].copy_from_slice(&value.to_ne_bytes());
        *offset += 2;
    }

    fn write_string(&mut self, str: &String, offset: &mut usize) {
        let mut str = str.clone();
        str.push('\0');
        let rounded_len: u32 = (str.len()+3) as u32 & (u32::MAX-3);
        self.write_u32(&rounded_len, offset);
        self[*offset..*offset+str.len()].copy_from_slice(str.as_bytes());
        *offset += rounded_len as usize;
    }

    fn read_u32(&self, offset: &mut usize) -> u32 {
        let res = u32::from_ne_bytes(
            self[*offset..*offset+4]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_u32")
        );
        *offset += 4;
        res
    }

    fn read_u16(&self, offset: &mut usize) -> u16 {
        let res = u16::from_ne_bytes(
            self[*offset..*offset+2]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_u32")
        );
        *offset += 2;
        res
    }

    fn read_string(&self, offset: &mut usize) -> String {
        let str_len = u32::from_ne_bytes(
            self[*offset..*offset+4]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_string")
        );
        *offset += 4;
        let str = String::from_utf8(
            self[*offset..*offset+((str_len-1) as usize)]
            .to_vec())
            .expect("String::from_utf8 failed in WlEvent::read_string()"
        );
        *offset += (str_len+3 & u32::MAX-3) as usize;
        str
    }
}

fn wl_connect() -> Result<UnixStream, Box<dyn Error>> {
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

fn wl_display_get_registry(wl_state: &mut WlState) -> Result<(), Box<dyn Error>> {
    const OBJECT: u32 = 1;
    const OPCODE: u16 = 1;
    const MSG_SIZE: u16 = 12;

    let mut request = [0u8; MSG_SIZE as usize];
    request[0..4].copy_from_slice(&OBJECT.to_ne_bytes());
    request[4..6].copy_from_slice(&OPCODE.to_ne_bytes());
    request[6..8].copy_from_slice(&MSG_SIZE.to_ne_bytes());

    wl_state.current_id += 1;
    request[8..12].copy_from_slice(&wl_state.current_id.to_ne_bytes());
    wl_state.registry_id = Some(wl_state.current_id);

    let written = wl_state.socket.write(&request)?;
    assert!(written == MSG_SIZE.into());

    Ok(())
}

fn wl_registry_bind(
    wl_state: &mut WlState,
    name: &u32,
    interface: &String,
    version: &u32,
    id: &u32
) -> Result<(), String> {
    let object: u32 = match wl_state.registry_id {
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

    match wl_state.socket.write(&request) {
        Ok(bytes) => {
            assert!(bytes == req_size as usize)
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    Ok(())
}

fn wl_shm_format(event: &Vec<u8>) {
    let mut offset = 0;
    println!("Received pixel format: {:x}", event.read_u32(&mut offset));
}

fn wl_shm_create_pool(wl_state: &mut WlState) -> Result<(), String> {
    wl_state.current_id += 1;
    wl_state.shm_pool = Some(match ShmPool::new(4096, wl_state.current_id) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    });

    let object = match wl_state.shm_id {
        Some(val) => val,
        None => {
            return Err("error in wl_shm_create_pool: shm_id not set!".to_string());
        }
    };
    const OPCODE: u16 = 0;
    const REQ_SIZE: u16 = 16;
    let id          = wl_state.shm_pool.as_ref().unwrap().id;
    let fds         = [wl_state.shm_pool.as_ref().unwrap().fd];
    let shm_size    = wl_state.shm_pool.as_ref().unwrap().size;

    let mut request = vec![0u8; REQ_SIZE as usize];
    let mut offset: usize = 0;

    // Request header
    request.write_u32(&object,          &mut offset);
    request.write_u16(&OPCODE,          &mut offset);
    request.write_u16(&REQ_SIZE,        &mut offset);

    // Id, size of shm pool
    request.write_u32(&id,                  &mut offset);
    request.write_u32(&(shm_size as u32),   &mut offset);
    
    println!("{:?}", request);

    let mut ancillary_buf = [0u8; 32];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buf[..]);
    assert!(ancillary.add_fds(&fds[..]));

    match wl_state.socket.send_vectored_with_ancillary(&[std::io::IoSlice::new(&request)], &mut ancillary) {
        Ok(bytes) => {
            assert!(bytes == REQ_SIZE as usize);
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    Ok(())
}

fn wl_shm_pool_resize(event: &Vec<u8>, wl_state: &mut WlState) -> std::io::Result<()> {
    let mut offset: usize = 0;
    let size = event.read_u32(&mut offset);
    wl_state.shm_pool.as_mut().unwrap().resize(size as usize)
}

fn wl_registry_global(event: &Vec<u8>, wl_state: &mut WlState) -> Result<(), Box<dyn Error>> {
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
        wl_state.current_id += 1;
        wl_registry_bind(
            wl_state,
            &name,
            &interface,
            &version,
            &wl_state.current_id.clone()
        )?;
        wl_state.shm_id = Some(wl_state.current_id);
        wl_shm_create_pool(wl_state)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    
    let wl_sock = match wl_connect() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("wl_connect failed: {}", err);
            return Err(err);
        }
    };

    let mut wl_state = WlState {
        socket:      wl_sock,
        current_id:  1, 
        registry_id: None,
        shm_id:      None,
        shm_pool:    None,
    };

    wl_display_get_registry(&mut wl_state)?;

    let mut event: Vec<u8> = Vec::new();
    let mut header = [0u8; 8];

    loop {
        wl_state.socket.read_exact(&mut header)?;

        let header = WlHeader {
            object: u32::from_ne_bytes(header[0..4].try_into()?),
            opcode: u16::from_ne_bytes(header[4..6].try_into()?),
            size:   u16::from_ne_bytes(header[6..8].try_into()?)
        };

        event.resize((header.size-8) as usize, 0);
        wl_state.socket.read_exact(&mut event)?;

        if header.object == wl_state.registry_id.unwrap() && header.opcode == 0 {
            wl_registry_global(&event, &mut wl_state)?;
        }
        else if header.object == 1 && header.opcode == 0 { // wl_display::error
            wl_display_error(&event);
        }
        else if wl_state.shm_id.is_some() && header.object == wl_state.shm_id.unwrap() && header.opcode == 0 { // wl_shm::format
            wl_shm_format(&event);
        }
        else if wl_state.shm_pool.is_some() && header.object == wl_state.shm_pool.as_ref().unwrap().id && header.opcode == 2 {
            wl_shm_pool_resize(&event, &mut wl_state)?;
        }
        else {
            println!(
                "Received event:\n\tObject: {}\n\tOpcode: {}\n\tSize: {}",
                header.object,
                header.opcode,
                header.size
            );
        }
        if false {
            break
        }
    }

    Ok(())
}
