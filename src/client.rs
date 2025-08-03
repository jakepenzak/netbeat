use crate::conf::NetbeatConf;
use spinners::{Spinner, Spinners};
use std::io;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::time::Instant;

pub fn contact(conf: NetbeatConf) -> io::Result<()> {
    let total_target_bytes = conf.data_size.unwrap() as usize;

    match TcpStream::connect(conf.socket_addr) {
        Ok(mut stream) => {
            println!("🌐 Connected to server at {}", conf.socket_addr);
            run_speed_test(&mut stream, conf.chunk_size as usize, total_target_bytes)?;
        }
        Err(e) => eprintln!("❌ Connection error: {}", e),
    }
    return Ok(());
}

fn run_speed_test(
    stream: &mut TcpStream,
    chunk_size: usize,
    total_target_bytes: usize,
) -> io::Result<()> {
    let mut buffer = vec![0; chunk_size];

    // Upload Test
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running upload speed test...".into());
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
    sp.stop();
    let upload_time = start_time.elapsed();
    let upload_seed_mbyte = (bytes_sent as f64 / 1e6) / (upload_time.as_secs_f64());
    println!("\n⏰ Upload complete in {:?}", upload_time);
    println!("⏫ Upload speed: {:.2} MB/s", upload_seed_mbyte);
    println!("⏫ Upload speed: {:.2} Mb/s\n", upload_seed_mbyte * 8.0);

    stream.shutdown(Shutdown::Write)?;

    // Download Test
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running download speed test...".into());
    let mut bytes_received: usize = 0;
    let start_time = Instant::now();

    while bytes_received < total_target_bytes {
        let remaining = total_target_bytes - bytes_received;
        let to_read = if remaining >= buffer.len() {
            buffer.len()
        } else {
            remaining
        };
        bytes_received += stream.read(&mut buffer[..to_read])?;
    }
    sp.stop();
    let download_time = start_time.elapsed();
    let download_speed_mbyte = (bytes_received as f64 / 1e6) / (download_time.as_secs_f64());
    println!("\n⏰ Download complete in {:?}", download_time);
    println!("⏬ Download speed: {:.2} MB/s", download_speed_mbyte);
    println!("⏬ Download speed: {:.2} Mb/s", download_speed_mbyte * 8.0);

    stream.shutdown(Shutdown::Read)?;
    Ok(())
}
