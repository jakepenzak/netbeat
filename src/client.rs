use crate::conf::NetbeatConf;
use crate::utils::generate_random_buffer;
use byte_unit::{Byte, UnitType};
use spinners::{Spinner, Spinners};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::{Duration, Instant};

pub fn contact(conf: NetbeatConf) -> std::io::Result<()> {
    match TcpStream::connect(conf.socket_addr) {
        Ok(mut stream) => {
            stream.set_nodelay(true)?;
            println!("🌐 Connected to server at {}\n", conf.socket_addr);
            run_speed_test(&mut stream, &conf)?;
        }
        Err(e) => eprintln!("❌ Connection error: {}", e),
    }
    return Ok(());
}

fn run_speed_test(stream: &mut TcpStream, conf: &NetbeatConf) -> std::io::Result<()> {
    let mut random_buffer = generate_random_buffer(conf.chunk_size as usize);
    let target_bytes = conf.data.unwrap();
    let time_secs = conf.time.unwrap_or(0);
    let target_time = Duration::from_secs(time_secs);
    let use_time = time_secs > 0;

    // Ping Test
    // TODO

    // Upload Test
    run_upload_test(
        stream,
        &mut random_buffer,
        target_bytes,
        target_time,
        use_time,
    )?;

    std::thread::sleep(Duration::from_millis(500));

    // Download Test
    run_download_test(
        stream,
        &mut random_buffer,
        target_bytes,
        target_time,
        use_time,
    )?;

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn run_upload_test(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    target_bytes: u64,
    target_time: Duration,
    use_time: bool,
) -> std::io::Result<()> {
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running upload speed test...".into());
    let mut bytes_sent: u64 = 0;

    let start_time = Instant::now();

    if use_time {
        // Time-based upload test
        while start_time.elapsed() < target_time {
            stream.write_all(&buffer)?;
            bytes_sent += buffer.len() as u64;
        }
    } else {
        // Byte-based upload test
        while bytes_sent < target_bytes {
            let remaining = target_bytes - bytes_sent;
            let to_write = if remaining >= buffer.len() as u64 {
                buffer.len() as u64
            } else {
                remaining
            };
            stream.write_all(&buffer[..to_write as usize])?;
            bytes_sent += to_write;
        }
    }
    sp.stop();
    let upload_time = start_time.elapsed();
    let upload_seed_megabyte = (bytes_sent as f64 / 1e6) / (upload_time.as_secs_f64());
    let unit = Byte::from_u64(bytes_sent).get_appropriate_unit(UnitType::Binary);
    println!("\n⏰ Uploaded {:.2} in {:.2?}", unit, upload_time);
    println!(
        "⏫ Upload speed: {:.2} MB/s, {:.2} Mb/s\n",
        upload_seed_megabyte,
        upload_seed_megabyte * 8.0
    );

    stream.write_all(b"UPLOAD_DONE")?;
    stream.flush()?;
    Ok(())
}

fn run_download_test(
    stream: &mut TcpStream,
    buffer: &mut Vec<u8>,
    target_bytes: u64,
    target_time: Duration,
    use_time: bool,
) -> std::io::Result<()> {
    let mut sp = Spinner::new(Spinners::Dots2, "🚀 Running download speed test...".into());
    let mut bytes_received: u64 = 0;
    let start_time = Instant::now();

    if use_time {
        // Time-base download test
        while start_time.elapsed() < target_time {
            match stream.read(buffer) {
                Ok(0) => break,
                Ok(n) => bytes_received += n as u64,
                Err(e) => {
                    sp.stop();
                    return Err(e);
                }
            }
        }
    } else {
        // Byte-base download test
        while bytes_received < target_bytes {
            let remaining = target_bytes - bytes_received;
            let to_read = if remaining >= buffer.len() as u64 {
                buffer.len() as u64
            } else {
                remaining
            };
            match stream.read(&mut buffer[..to_read as usize]) {
                Ok(0) => break,
                Ok(n) => bytes_received += n as u64,
                Err(e) => {
                    sp.stop();
                    return Err(e);
                }
            }
        }
    }
    sp.stop();
    let download_time = start_time.elapsed();
    let download_speed_megabyte = (bytes_received as f64 / 1e6) / (download_time.as_secs_f64());
    let unit = Byte::from_u64(bytes_received).get_appropriate_unit(UnitType::Binary);
    println!("\n⏰ Downloaded {:.2} in {:.2?}", unit, download_time);
    println!(
        "⏬ Download speed: {:.2} MB/s, {:.2} Mb/s\n",
        download_speed_megabyte,
        download_speed_megabyte * 8.0
    );
    Ok(())
}
