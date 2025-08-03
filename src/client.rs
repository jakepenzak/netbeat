use crate::conf::NetbeatConf;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Instant;

pub fn contact(conf: NetbeatConf) -> io::Result<()> {
    let total_target_bytes = conf.data_size.unwrap() as usize;
    let send_buffer = vec![0; conf.chunk_size as usize];

    match TcpStream::connect(conf.socket_addr) {
        Ok(mut stream) => {
            println!("Connected to server at {}", conf.socket_addr);
            run_speed_test(&mut stream, send_buffer, total_target_bytes)?;
        }
        Err(e) => eprintln!("Connection error: {}", e),
    }
    Ok(())
}

fn run_speed_test(
    stream: &mut TcpStream,
    buffer: Vec<u8>,
    total_target_bytes: usize,
) -> io::Result<()> {
    let mut bytes_sent: usize = 0;
    let start_time = Instant::now();

    while bytes_sent < total_target_bytes {
        let remaining = total_target_bytes - bytes_sent;
        let to_write = if remaining >= buffer.len() {
            buffer.len()
        } else {
            remaining
        };
        stream.write_all(&buffer[..to_write])?;
        bytes_sent += to_write;
    }

    let upload_time = start_time.elapsed();
    println!("Upload complete in {:?}", upload_time);
    println!(
        "Upload speed: {:.2} MB/s",
        (bytes_sent as f64 / 1e6) / (upload_time.as_secs_f64())
    );

    // // Download Test
    // let mut download_buffer = vec![0; buffer.len()];
    // let start = Instant::now();
    // stream.read_exact(&mut download_buffer)?;
    // let download_time = start.elapsed();
    // println!("Download complete in {:?}", download_time);
    // println!(
    //     "Download speed: {:.2} MB/s",
    //     buffer.len() as f64 / download_time.as_millis() as f64 / 1000.0
    // );
    Ok(())
}
