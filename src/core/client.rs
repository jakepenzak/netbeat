use super::{config, protocol};
use crate::{
    output::reports::{self, NetbeatReport, PingReport, Report, SpeedReport},
    utils::Logger,
};

use byte_unit::Byte;
use spinners::{Spinner, Spinners};
use std::{
    error::Error as StdError,
    io::{self, Read},
    net::{IpAddr, Shutdown, SocketAddr, TcpStream},
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Client {
    pub socket_addr: SocketAddr,
    pub data: u64,
    pub time: u64,
    pub chunk_size: u64,
    pub ping_count: u32,
    pub return_json: bool,
    pub timeout: Duration,
    pub retries: u32,
    pub logger: Logger,
}

impl Client {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target: String,
        port: Option<u16>,
        data: Option<&str>,
        time: Option<u64>,
        chunk_size: Option<&str>,
        ping_count: Option<u32>,
        return_json: Option<bool>,
        timeout: Option<u64>,
        retries: Option<u32>,
        quiet: Option<bool>,
        verbose: Option<bool>,
    ) -> Result<Self, Box<dyn StdError>> {
        Ok(Self {
            socket_addr: SocketAddr::new(
                IpAddr::from_str(&target)?,
                port.unwrap_or(config::DEFAULT_PORT),
            ),
            data: Byte::parse_str(data.unwrap_or(config::DEFAULT_TARGET_DATA), false)?.as_u64(),
            time: time.unwrap_or(config::DEFAULT_TEST_DURATION),
            chunk_size: Byte::parse_str(chunk_size.unwrap_or(config::DEFAULT_CHUNK_SIZE), false)?
                .as_u64(),
            ping_count: ping_count.unwrap_or(config::DEFAULT_PING_COUNT),
            return_json: return_json.unwrap_or(false),
            timeout: Duration::from_secs(timeout.unwrap_or(config::DEFAULT_CONNECTION_TIMEOUT)),
            retries: retries.unwrap_or(config::DEFAULT_MAX_RETRIES),
            logger: Logger::new(quiet.unwrap_or(false), verbose.unwrap_or(false)),
        })
    }

    pub fn contact(&self) -> io::Result<NetbeatReport> {
        let mut last_error = None;

        for attempt in 1..=self.retries {
            match TcpStream::connect_timeout(&self.socket_addr, self.timeout) {
                Ok(mut stream) => {
                    stream.set_nodelay(true)?;
                    stream.set_write_timeout(Some(self.timeout))?;
                    stream.set_read_timeout(Some(self.timeout))?;
                    eprintln!("🌐 Connected to server at {}\n", self.socket_addr);

                    let netbeat_report = self.run_speed_test(&mut stream)?;
                    return Ok(netbeat_report);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.retries {
                        continue;
                    }
                }
            }
        }
        Err(last_error.unwrap())
    }

    fn run_speed_test(&self, stream: &mut TcpStream) -> io::Result<NetbeatReport> {
        let mut random_buffer = protocol::generate_random_buffer(self.chunk_size as usize);
        let target_bytes = self.data;
        let target_time = Duration::from_secs(self.time);
        let use_time = target_bytes == 0;

        // Ping Test
        let ping_report = self.run_ping_test(stream)?;

        // Upload Test
        let upload_report = self.run_upload_test(
            stream,
            &mut random_buffer,
            target_bytes,
            target_time,
            use_time,
        )?;

        thread::sleep(Duration::from_millis(500));

        // Download Test
        let download_report = self.run_download_test(
            stream,
            &mut random_buffer,
            target_bytes,
            target_time,
            use_time,
        )?;

        let netbeat_report = NetbeatReport::new(ping_report, upload_report, download_report);

        eprintln!("{}", netbeat_report.table_report());

        if self.return_json {
            println!("{}", netbeat_report.to_json());
        }
        stream.shutdown(Shutdown::Both)?;
        Ok(netbeat_report)
    }

    fn run_ping_test(&self, stream: &mut TcpStream) -> io::Result<PingReport> {
        let msg = "🏓 Running ping test...";
        let mut sp = Spinner::new(Spinners::Dots2, msg.into());

        let mut ping_buffer = [0u8; protocol::PING_RESPONSE.len()];
        let mut ping_times: Vec<Duration> = Vec::with_capacity(self.ping_count as usize);
        let mut successful_pings = 0;

        // Send initial ping
        protocol::write_message(stream, protocol::PING_MESSAGE)?;

        // Ping test
        for i in 0..self.ping_count {
            let start_time = Instant::now();

            match protocol::write_message(stream, protocol::PING_MESSAGE) {
                Ok(_) => match stream.read_exact(&mut ping_buffer) {
                    Ok(_) => {
                        let ping_time = start_time.elapsed();
                        if ping_buffer == protocol::PING_RESPONSE {
                            successful_pings += 1;
                            ping_times.push(ping_time);
                        }
                    }
                    Err(_) => continue,
                },
                Err(_) => continue,
            }

            if i < self.ping_count - 1 {
                thread::sleep(Duration::from_millis(100));
            }
        }

        sp.stop_with_message(format!("{msg} ✅ Completed."));

        // Send close message
        protocol::write_message(stream, protocol::PING_DONE)?;

        // Report
        let ping_report = PingReport::new(self.ping_count, successful_pings, ping_times);
        if successful_pings > 0 {
            eprintln!("{}", ping_report.table_report());
        } else {
            eprintln!("\n❌ Ping test failed - no successful responses received\n");
        }

        Ok(ping_report)
    }

    #[allow(clippy::collapsible_if)]
    fn run_upload_test(
        &self,
        stream: &mut TcpStream,
        buffer: &mut [u8],
        target_bytes: u64,
        target_time: Duration,
        use_time: bool,
    ) -> io::Result<SpeedReport> {
        let msg = "🚀 Running upload speed test...";
        let mut sp = Spinner::new(Spinners::Dots2, msg.into());
        let mut bytes_sent: u64 = 0;

        let start_time = Instant::now();

        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs(1);
        let mut iteration_count = 0u64;
        let check_interval = 500;

        // Send initial upload start
        protocol::write_message(stream, protocol::UPLOAD_START)?;

        // Upload test
        if use_time {
            // Time-based upload test
            while start_time.elapsed() < target_time {
                protocol::write_message(stream, buffer)?;
                bytes_sent += buffer.len() as u64;
                iteration_count += 1;
                if iteration_count % check_interval == 0 {
                    if last_update.elapsed() >= update_interval {
                        sp =
                            reports::print_progress(start_time.elapsed(), bytes_sent, &mut sp, msg);
                        last_update = Instant::now();
                    }
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
                protocol::write_message(stream, &buffer[..to_write as usize])?;
                bytes_sent += to_write;
                iteration_count += 1;
                if iteration_count % check_interval == 0 {
                    if last_update.elapsed() >= update_interval {
                        sp =
                            reports::print_progress(start_time.elapsed(), bytes_sent, &mut sp, msg);
                        last_update = Instant::now();
                    }
                }
            }
        }
        let upload_time = start_time.elapsed();
        sp.stop_with_message(format!("{msg} ✅ Completed."));

        // Send close message
        protocol::write_message(stream, protocol::UPLOAD_DONE)?;

        // Report
        let upload_report = SpeedReport::new("upload", upload_time, bytes_sent).unwrap();
        eprintln!("{}", upload_report.table_report());
        Ok(upload_report)
    }

    #[allow(clippy::collapsible_if)]
    fn run_download_test(
        &self,
        stream: &mut TcpStream,
        buffer: &mut [u8],
        target_bytes: u64,
        target_time: Duration,
        use_time: bool,
    ) -> io::Result<SpeedReport> {
        let msg = "🚀 Running download speed test...";
        let mut sp = Spinner::new(Spinners::Dots2, msg.into());
        let mut bytes_received: u64 = 0;
        let start_time = Instant::now();

        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs(1);
        let mut iteration_count = 0u64;
        let check_interval = 500;

        // Send initial download start
        protocol::write_message(stream, protocol::DOWNLOAD_START)?;

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
                iteration_count += 1;
                if iteration_count % check_interval == 0 {
                    if last_update.elapsed() >= update_interval {
                        sp = reports::print_progress(
                            start_time.elapsed(),
                            bytes_received,
                            &mut sp,
                            msg,
                        );
                        last_update = Instant::now();
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
                iteration_count += 1;
                if iteration_count % check_interval == 0 {
                    if last_update.elapsed() >= update_interval {
                        sp = reports::print_progress(
                            start_time.elapsed(),
                            bytes_received,
                            &mut sp,
                            msg,
                        );
                        last_update = Instant::now();
                    }
                }
            }
        }
        let download_time = start_time.elapsed();
        sp.stop_with_message(format!("{msg} ✅ Completed."));

        // Send close message
        // protocol::write_message(stream, protocol::DOWNLOAD_DONE)?;

        // Report
        let download_report = SpeedReport::new("download", download_time, bytes_received).unwrap();
        eprintln!("{}", download_report.table_report());
        Ok(download_report)
    }
}
