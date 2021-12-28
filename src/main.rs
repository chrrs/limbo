use std::net::TcpListener;

use anyhow::{Context, Result};

mod protocol;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9010").context("failed to open TCP listener")?;

    for _ in listener.incoming() {
        println!("new connection!")
    }

    Ok(())
}
