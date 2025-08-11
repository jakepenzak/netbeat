use crate::conf::NetbeatConf;
use crate::reports::print_progress;
use crate::utils::{PING_MESSAGE, PING_RESPONSE, PING_TERMINATOR, generate_random_buffer};
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
        Err(e) => eprintln!("❌ Connection error: {e}"),
    }
    Ok(())
}

fn run_speed_test(stream: &mut TcpStream, conf: &NetbeatConf) -> std::io::Result<()> {
    let mut random_buffer = generate_random_buffer(conf.chunk_size as usize);
    let target_bytes = conf.data.unwrap();
    let time_secs = conf.time.unwrap_or(0);
    let target_time = Duration::from_secs(time_secs);
    let use_time = target_bytes == 0;

    // Ping Test
    run_ping_test(stream, conf.ping_count.unwrap())?;

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

fn run_ping_test(stream: &mut TcpStream, ping_count: u32) -> std::io::Result<()> {
    let msg = "🏓 Running ping test ...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());

    let mut ping_buffer = [0u8; 4];
    let mut ping_times: Vec<Duration> = Vec::with_capacity(ping_count as usize);
    let mut successful_pings = 0;

    // Send initial ping
    stream.write_all(b"DING")?;
    stream.flush()?;

    // Ping test
    for i in 0..ping_count {
        let start_time = Instant::now();

        match stream.write_all(PING_MESSAGE) {
            Ok(_) => {
                stream.flush()?;

                stream.set_read_timeout(Some(Duration::from_secs(3)))?;

                match stream.read_exact(&mut ping_buffer) {
                    Ok(_) => {
                        let ping_time = start_time.elapsed();
                        if ping_buffer == PING_RESPONSE {
                            successful_pings += 1;
                            ping_times.push(ping_time);
                        }
                    }
                    Err(_) => continue,
                }
            }
            Err(_) => continue,
        }

        if i < ping_count - 1 {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    stream.set_read_timeout(None)?;
    stream.write_all(PING_TERMINATOR)?;
    stream.flush()?;

    sp.stop_with_message(format!("{msg} --> ✅ Completed."));

    if successful_pings > 0 {
        let min_ping = ping_times.iter().min().unwrap();
        let max_ping = ping_times.iter().max().unwrap();
        let avg_ping = ping_times.iter().sum::<Duration>() / ping_times.len() as u32;
        let packet_loss = ((ping_count - successful_pings) as f64 / ping_count as f64) * 100.0;

        println!("\n🏓 Ping Statistics:");
        println!("   📊 Packets sent: {ping_count}, Packets received: {successful_pings}");
        println!("   📉 Packet loss: {packet_loss:.1}%");
        println!("   ▪️ Min RTT: {min_ping:.2?}");
        println!("   ⬛ Max RTT: {max_ping:.2?}");
        println!("   ◾ Avg RTT: {avg_ping:.2?}\n");
    } else {
        println!("\n❌ Ping test failed - no successful responses received\n");
    }

    Ok(())
}

fn run_upload_test(
    stream: &mut TcpStream,
    buffer: &mut [u8],
    target_bytes: u64,
    target_time: Duration,
    use_time: bool,
) -> std::io::Result<()> {
    let msg = "🚀 Running upload speed test...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());
    let mut bytes_sent: u64 = 0;

    let start_time = Instant::now();

    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1);

    if use_time {
        // Time-based upload test
        while start_time.elapsed() < target_time {
            stream.write_all(buffer)?;
            bytes_sent += buffer.len() as u64;
            if last_update.elapsed() >= update_interval {
                sp = print_progress(start_time.elapsed(), bytes_sent, &mut sp, msg);
                last_update = Instant::now();
            }
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
            if last_update.elapsed() >= update_interval {
                sp = print_progress(start_time.elapsed(), bytes_sent, &mut sp, msg);
                last_update = Instant::now();
            }
        }
    }
    sp.stop_with_message(format!("{msg} --> ✅ Completed."));

    let upload_time = start_time.elapsed();
    let upload_seed_megabyte = (bytes_sent as f64 / 1e6) / (upload_time.as_secs_f64());
    let unit = Byte::from_u64(bytes_sent).get_appropriate_unit(UnitType::Binary);
    println!("\n⏰ Uploaded {unit:.2} in {upload_time:.2?}");
    println!(
        "⏫ Upload speed: {:.2} MiB/s, {:.2} Mib/s\n",
        upload_seed_megabyte,
        upload_seed_megabyte * 8.0
    );

    stream.write_all(b"UPLOAD_DONE")?;
    stream.flush()?;
    Ok(())
}

fn run_download_test(
    stream: &mut TcpStream,
    buffer: &mut [u8],
    target_bytes: u64,
    target_time: Duration,
    use_time: bool,
) -> std::io::Result<()> {
    let msg = "🚀 Running download speed test...";
    let mut sp = Spinner::new(Spinners::Dots2, msg.into());
    let mut bytes_received: u64 = 0;
    let start_time = Instant::now();

    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1);

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
            if last_update.elapsed() >= update_interval {
                sp = print_progress(start_time.elapsed(), bytes_received, &mut sp, msg);
                last_update = Instant::now();
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
            if last_update.elapsed() >= update_interval {
                sp = print_progress(start_time.elapsed(), bytes_received, &mut sp, msg);
                last_update = Instant::now();
            }
        }
    }
    sp.stop_with_message(format!("{msg} --> ✅ Completed."));

    let download_time = start_time.elapsed();
    let download_speed_megabyte = (bytes_received as f64 / 1e6) / (download_time.as_secs_f64());
    let unit = Byte::from_u64(bytes_received).get_appropriate_unit(UnitType::Binary);
    println!("\n⏰ Downloaded {unit:.2} in {download_time:.2?}");
    println!(
        "⏬ Download speed: {:.2} MiB/s, {:.2} Mib/s\n",
        download_speed_megabyte,
        download_speed_megabyte * 8.0
    );
    Ok(())
}
