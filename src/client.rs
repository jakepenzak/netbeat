use crate::conf::NetbeatConf;
use crate::utils::generate_random_buffer;
use byte_unit::Byte;
use spinners::{Spinner, Spinners};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Instant;

pub fn contact(conf: NetbeatConf) -> std::io::Result<()> {
    let total_target_bytes = conf.data_size.unwrap();

    match TcpStream::connect(conf.socket_addr) {
        Ok(mut stream) => {
            println!("🌐 Connected to server at {}\n", conf.socket_addr);
            run_speed_test(&mut stream, conf.chunk_size, total_target_bytes)?;
        }
        Err(e) => eprintln!("❌ Connection error: {}", e),
    }
    return Ok(());
}

fn run_speed_test(
    stream: &mut TcpStream,
    chunk_size: u64,
    total_target_bytes: u64,
) -> std::io::Result<()> {
    let mut random_buffer = generate_random_buffer(chunk_size as usize);

    // Upload Test
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running upload speed test...".into());
    let mut bytes_sent: u64 = 0;
    let start_time = Instant::now();

    while bytes_sent < total_target_bytes {
        let remaining = total_target_bytes - bytes_sent;
        let to_write = if remaining >= random_buffer.len() as u64 {
            random_buffer.len() as u64
        } else {
            remaining
        };
        stream.write_all(&random_buffer[..to_write as usize])?;
        bytes_sent += to_write;
    }
    sp.stop();
    let upload_time = start_time.elapsed();
    let upload_seed_mbyte = (bytes_sent as f64 / 1e6) / (upload_time.as_secs_f64());
    let (upload, unit) = Byte::from_u64(bytes_sent).get_exact_unit(false);
    println!("\n⏰ Uploaded {} {} in {:.2?}", upload, unit, upload_time);
    println!(
        "⏫ Upload speed: {:.2} MB/s, {:.2} Mb/s\n",
        upload_seed_mbyte,
        upload_seed_mbyte * 8.0
    );

    stream.shutdown(Shutdown::Write)?;

    // Download Test
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running download speed test...".into());
    let mut bytes_received: u64 = 0;
    let start_time = Instant::now();

    while bytes_received < total_target_bytes {
        let remaining = total_target_bytes - bytes_received;
        let to_read = if remaining >= random_buffer.len() as u64 {
            random_buffer.len() as u64
        } else {
            remaining
        };
        bytes_received += stream.read(&mut random_buffer[..to_read as usize])? as u64;
    }
    sp.stop();
    let download_time = start_time.elapsed();
    let download_speed_mbyte = (bytes_received as f64 / 1e6) / (download_time.as_secs_f64());
    let (download, unit) = Byte::from_u64(bytes_received).get_exact_unit(false);
    println!(
        "\n⏰ Downloaded {} {} in {:.2?}",
        download, unit, download_time
    );
    println!(
        "⏬ Download speed: {:.2} MB/s, {:.2} Mb/s\n",
        download_speed_mbyte,
        download_speed_mbyte * 8.0
    );

    stream.shutdown(Shutdown::Read)?;
    Ok(())
}
