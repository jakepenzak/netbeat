use crate::conf::NetbeatConf;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

pub fn contact(conf: NetbeatConf) -> io::Result<()> {
    let mut buffer = vec![0; conf.data_size.unwrap() as usize];
    println!("Connecting to server...");

    match TcpStream::connect(conf.socket_addr) {
        Ok(mut stream) => {
            run_speed_test(&mut stream, &mut buffer)?;
        }
        Err(e) => eprintln!("Connection error: {}", e),
    }
    Ok(())
}

fn run_speed_test(stream: &mut TcpStream, buffer: &[u8]) -> io::Result<()> {
    // Upload Test
    let start = Instant::now();
    stream.write_all(buffer)?;
    let upload_time = start.elapsed();
    println!("Upload complete in {:?}", upload_time);
    println!(
        "Upload speed: {:.2} MB/s",
        buffer.len() as f64 / upload_time.as_millis() as f64 / 1000.0
    );

    // Download Test
    let mut download_buffer = vec![0; buffer.len()];
    let start = Instant::now();
    stream.read_exact(&mut download_buffer)?;
    let download_time = start.elapsed();
    println!("Download complete in {:?}", download_time);
    println!(
        "Download speed: {:.2} MB/s",
        buffer.len() as f64 / download_time.as_millis() as f64 / 1000.0
    );
    Ok(())
}
