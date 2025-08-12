use crate::utils;
use byte_unit::Byte;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Server {
    pub socket_addr: SocketAddr,
    pub chunk_size: u64,
}

impl Server {
    pub fn new(target: String, port: u16, chunk_size: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            chunk_size: Byte::parse_str(chunk_size, false)?.as_u64(),
        })
    }

    pub fn listen(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.socket_addr)?;
        println!(
            "🌐 Server Listening on {}",
            listener.local_addr().unwrap().port()
        );

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    stream.set_nodelay(true)?;
                    println!("\n🌐 New connection from {}", stream.peer_addr()?);
                    let chunk_size = self.chunk_size;
                    thread::spawn(move || handle_client(stream, chunk_size));
                }
                Err(e) => println!("❌ Connection failed: {e}"),
            }
        }
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, chunk_size: u64) -> std::io::Result<()> {
    // Ping Test
    handle_ping_test(&mut stream)?;

    std::thread::sleep(Duration::from_millis(50));

    // Upload Test
    handle_upload_test(&mut stream, chunk_size)?;

    std::thread::sleep(Duration::from_millis(50));

    // Download Test
    handle_download_test(&mut stream, chunk_size)?;

    Ok(())
}

fn handle_ping_test(stream: &mut TcpStream) -> std::io::Result<()> {
    let msg = "🏓 Running ping test for client...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());

    let mut ping_buffer = [0u8; 4];

    loop {
        match stream.read_exact(&mut ping_buffer) {
            Ok(_) => {
                if ping_buffer == utils::PING_TERMINATOR {
                    break;
                } else if ping_buffer == utils::PING_MESSAGE {
                    stream.write_all(utils::PING_RESPONSE)?;
                } else {
                    continue;
                }
            }
            Err(e) => {
                eprintln!("❌ Error reading from client: {e}");
                break;
            }
        }
    }
    sp.stop_with_message(format!("{msg} ✅ Completed."));
    Ok(())
}

fn handle_upload_test(stream: &mut TcpStream, chunk_size: u64) -> std::io::Result<()> {
    let mut buffer = vec![0u8; chunk_size as usize];

    let msg = "🚀 Running upload speed test for client...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if n >= 11 && &buffer[n - 11..n] == b"UPLOAD_DONE" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Error reading from client: {e}");
                break;
            }
        }
    }
    sp.stop_with_message(format!("{msg} ✅ Completed."));
    Ok(())
}

fn handle_download_test(stream: &mut TcpStream, chunk_size: u64) -> std::io::Result<()> {
    let random_buffer = utils::generate_random_buffer(chunk_size as usize);

    let msg = "🚀 Running download speed test for client...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());
    loop {
        match stream.write_all(&random_buffer) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::BrokenPipe => break,
                _ => {
                    eprintln!("❌ Error writing to client: {e}");
                    break;
                }
            },
        }
    }
    sp.stop_with_message(format!("{msg} ✅ Completed."));
    Ok(())
}
