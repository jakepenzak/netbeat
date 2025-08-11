use crate::conf::NetbeatConf;
use crate::utils::{PING_MESSAGE, PING_RESPONSE, PING_TERMINATOR, generate_random_buffer};
use spinners::{Spinner, Spinners};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub fn listen(conf: NetbeatConf) -> std::io::Result<()> {
    let listener = TcpListener::bind(conf.socket_addr)?;
    println!(
        "🌐 Server Listening on {}",
        listener.local_addr().unwrap().port()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("\n🌐 New connection from {}", stream.peer_addr()?);
                thread::spawn(move || handle_client(stream, conf.chunk_size));
            }
            Err(e) => println!("❌ Connection failed: {e}"),
        }
    }
    Ok(())
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
                if ping_buffer == PING_TERMINATOR {
                    break;
                } else if ping_buffer == PING_MESSAGE {
                    stream.write_all(PING_RESPONSE)?;
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
    sp.stop_with_message(format!("{msg} --> ✅ Completed."));
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
    sp.stop_with_message(format!("{msg} --> ✅ Completed."));
    Ok(())
}

fn handle_download_test(stream: &mut TcpStream, chunk_size: u64) -> std::io::Result<()> {
    let random_buffer = generate_random_buffer(chunk_size as usize);

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
    sp.stop_with_message(format!("{msg} --> ✅ Completed."));
    Ok(())
}
