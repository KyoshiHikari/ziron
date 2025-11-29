//! Ziron Prompt - Shell prompt binary that communicates with the daemon

use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use ziron_core::error::Result;

fn get_socket_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| ziron_core::error::Error::Config("HOME not set".to_string()))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ziron")
        .join("ziron.sock"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let socket_path = get_socket_path()?;

    // Connect to daemon
    let mut stream = match UnixStream::connect(&socket_path).await {
        Ok(stream) => stream,
        Err(_) => {
            // Daemon not running, return empty prompt
            eprintln!("Warning: Ziron daemon not running. Start it with: ziron-daemon");
            std::process::exit(0);
        }
    };

    // Send request (empty for now, daemon will use environment)
    stream.write_all(b"prompt").await?;
    stream.shutdown().await?;

    // Read response
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    // Output prompt
    print!("{}", String::from_utf8_lossy(&buffer));

    Ok(())
}

