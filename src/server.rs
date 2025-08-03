use crate::conf::NetbeatConf;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn listen(conf: NetbeatConf) -> std::io::Result<()> {
    let listener = TcpListener::bind(conf.socket_addr)?;
    println!("Listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from {}", stream.peer_addr()?);
                thread::spawn(move || handle_client(stream, conf.buffer_size.unwrap()));
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, buffer_size: u64) -> std::io::Result<()> {
    let mut buffer = vec![0; buffer_size as usize];

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        stream.write(&buffer[..bytes_read])?;
    }
    Ok(())
}
