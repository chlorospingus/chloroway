#![feature(unix_socket_ancillary_data)]
use std::error::Error;

mod wl_client;
mod wl_registry;
mod wl_shm;
mod vec_utils;
mod shm;
mod surface;
use wl_client::WlClient;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wl_client = WlClient::new()?;

    wl_client.wl_display_get_registry()?;

    loop {
        wl_client.read_event()?;

        if false {
            break
        }
    }

    Ok(())
}
