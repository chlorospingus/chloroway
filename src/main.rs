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

fn main() ->Result<(), Box<dyn Error>> {
    
    let mut wl_sock = match wl_connect() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("wl_connect failed: {}", err);
            return Err(err);
        }
    };

    Ok(())
}
