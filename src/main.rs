#![feature(unix_socket_ancillary_data)]
use std::{error::Error, thread, time::Duration};

mod wayland;
use wayland::wl_client::WlClient;
mod graphics;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wl_client = WlClient::run()?;

    Ok(())
}
