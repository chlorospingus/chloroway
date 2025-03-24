use std::{env, error::Error, os::unix::net::UnixStream};


fn wl_connect() -> Result<UnixStream, Box<dyn Error>> {
    let wl_sock_path: String = format!(
        "{}/{}",
        env::var("XDG_RUNTIME_DIR")?,
        env::var("WAYLAND_DISPLAY")?
    );
    let sock = UnixStream::connect(wl_sock_path)?;

    Ok(sock)
}


fn wl_display_get_registry(wl_sock: &mut UnixStream, wl_current_id: &mut u32) -> Result<(), Box<dyn Error>> {
    const OBJECT: u32 = 1;
    const OPCODE: u16 = 1;
    const MSG_SIZE: u16 = 12;

    let mut request = [0u8; MSG_SIZE as usize];
    request[0..4].copy_from_slice(&OBJECT.to_ne_bytes());
    request[4..6].copy_from_slice(&OPCODE.to_ne_bytes());
    request[6..8].copy_from_slice(&MSG_SIZE.to_ne_bytes());

    *wl_current_id += 1;
    request[8..12].copy_from_slice(&wl_current_id.to_ne_bytes());

    let written = wl_sock.write(&request)?;
    assert!(written == MSG_SIZE.into());

    Ok(())
}

fn main() ->Result<(), Box<dyn Error>> {
    
    let mut wl_current_id: u32 = 0;

    let mut wl_sock = match wl_connect() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("wl_connect failed: {}", err);
            return Err(err);
        }
    };

    wl_display_get_registry(&mut wl_sock, &mut wl_current_id);

    Ok(())
}
