use crate::conf::NetbeatConf;
use spinners::{Spinner, Spinners};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn listen(conf: NetbeatConf) -> std::io::Result<()> {
    let listener = TcpListener::bind(conf.socket_addr)?;
    println!("🌐 Listening on {}\n", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("🌐 New connection from {}", stream.peer_addr()?);
                thread::spawn(move || handle_client(stream, conf.chunk_size));
            }
            Err(e) => println!("❌ Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, chunk_size: u64) -> std::io::Result<()> {
    let mut buffer = vec![0u8; chunk_size as usize];

    let mut sp = Spinner::new(
        Spinners::Dots2,
        "🚀 Running upload speed test for client...".into(),
    );
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ Error reading from client: {}", e);
                break;
            }
        }
    }
    println!("\n✅ Completed.");
    sp.stop();

    let mut sp = Spinner::new(
        Spinners::Dots2,
        "🚀 Running download speed test for client...".into(),
    );
    loop {
        match stream.write(&buffer) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::BrokenPipe => break,
                _ => {
                    eprintln!("❌ Error writing to client: {}", e);
                    break;
                }
            },
        }
    }

    println!("\n✅ Completed.");
    sp.stop();
    Ok(())
}
