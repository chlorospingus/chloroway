use std::{env, error::Error, io::{Read, Write}, os::unix::net::UnixStream};

struct WlState {
    socket: UnixStream,
    current_id: u32,
    registry_id: u32,
}

struct WlHeader {
    object: u32,
    opcode: u16,
    size: u16
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
    wl_state.registry_id = wl_state.current_id;

    let written = wl_state.socket.write(&request)?;
    assert!(written == MSG_SIZE.into());

    Ok(())
}

fn wl_registry_global(header: &WlHeader, wl_sock: &mut UnixStream) {
    
}

fn main() ->Result<(), Box<dyn Error>> {
    
    let wl_sock = match wl_connect() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("wl_connect failed: {}", err);
            return Err(err);
        }
    };

    let mut wl_state = WlState {
        socket: wl_sock,
        current_id: 0, 
        registry_id: 0,
    };

    wl_display_get_registry(&mut wl_state)?;

    let mut header = [0u8; 8];
    wl_state.socket.read_exact(&mut header)?;
    let header = WlHeader {
        object: u32::from_ne_bytes(header[0..4].try_into().unwrap()),
        opcode: u16::from_ne_bytes(header[4..6].try_into().unwrap()),
        size: u16::from_ne_bytes(header[6..8].try_into().unwrap())
    };

    println!(
        "Object: {}\nOpcode: {}\nSize: {}",
        header.object,
        header.opcode,
        header.size
    );

    Ok(())
}
