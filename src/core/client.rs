use super::{config, protocol};
use crate::{
    output::reports::{self, NetbeatReport, PingReport, Report, SpeedReport},
    utils::{
        error::{NetbeatError, Result},
        logging::Logger,
    },
};

use byte_unit::Byte;
use spinners::{Spinner, Spinners};
use std::{
    io::{ErrorKind, Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, TcpStream},
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Client {
    pub socket_addr: SocketAddr,
    pub data: Option<u64>,
    pub time: u64,
    pub chunk_size: u64,
    pub ping_count: u32,
    pub return_json: bool,
    pub timeout: Duration,
    pub retries: u32,
    pub logger: Logger,
}

#[derive(Debug)]
pub struct ClientBuilder {
    target: String,
    port: Option<u16>,
    data: Option<String>,
    time: Option<u64>,
    chunk_size: Option<String>,
    ping_count: Option<u32>,
    return_json: Option<bool>,
    timeout: Option<u64>,
    retries: Option<u32>,
    quiet: Option<bool>,
    verbose: Option<bool>,
}

impl Client {
    pub fn builder(target: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(target)
    }

    pub fn contact(&self) -> Result<NetbeatReport> {
        for attempt in 1..=self.retries {
            match TcpStream::connect_timeout(&self.socket_addr, self.timeout) {
                Ok(mut stream) => {
                    stream
                        .set_nodelay(true)
                        .map_err(NetbeatError::ConnectionError)?;
                    stream
                        .set_write_timeout(Some(self.timeout))
                        .map_err(NetbeatError::ConnectionError)?;
                    stream
                        .set_read_timeout(Some(self.timeout))
                        .map_err(NetbeatError::ConnectionError)?;

                    self.logger
                        .info(&format!("🌐 Connected to server at {}\n", self.socket_addr));

                    return self.run_speed_test(&mut stream);
                }
                Err(_) if attempt < self.retries => continue,
                Err(e) => {
                    return Err(NetbeatError::ConnectionError(e));
                }
            }
        }
        Err(NetbeatError::client(
            "All connection attempts failed".to_string(),
        ))
    }

    fn run_speed_test(&self, stream: &mut TcpStream) -> Result<NetbeatReport> {
        let mut random_buffer = protocol::generate_random_buffer(self.chunk_size as usize);
        let target_bytes = self.data;
        let target_time = Duration::from_secs(self.time);
        let use_time = target_bytes.is_none();

        // Ping Test
        let ping_report = self
            .run_ping_test(stream)
            .map_err(|e| NetbeatError::test_execution(format!("Ping test failed - {e}")))?;

        // Upload Test
        let upload_report = self
            .run_upload_test(
                stream,
                &mut random_buffer,
                target_bytes,
                target_time,
                use_time,
            )
            .map_err(|e| NetbeatError::test_execution(format!("Upload test failed - {e}")))?;

        thread::sleep(Duration::from_millis(500));

        // Download Test
        let download_report = self
            .run_download_test(
                stream,
                &mut random_buffer,
                target_bytes,
                target_time,
                use_time,
            )
            .map_err(|e| NetbeatError::test_execution(format!("Download test failed - {e}")))?;

        let netbeat_report = NetbeatReport::new(ping_report, upload_report, download_report);

        self.logger
            .info(&format!("{}", netbeat_report.table_report()));

        if self.return_json {
            self.logger.result(&format!("{}", netbeat_report.to_json()));
        }
        stream
            .shutdown(Shutdown::Both)
            .map_err(NetbeatError::ConnectionError)?;
        Ok(netbeat_report)
    }

    fn run_ping_test(&self, stream: &mut TcpStream) -> Result<PingReport> {
        let msg = "🏓 Running ping test...";
        let sp = if !self.logger.quiet & !self.logger.verbose {
            Some(Spinner::new(Spinners::Dots2, msg.into()))
        } else {
            None
        };
        self.logger.verbose(msg);

        let mut ping_buffer = [0u8; protocol::PING_RESPONSE.len()];
        let mut ping_times: Vec<Duration> = Vec::with_capacity(self.ping_count as usize);
        let mut successful_pings = 0;

        // Send initial ping
        protocol::write_message(stream, protocol::PING_MESSAGE)
            .map_err(|e| NetbeatError::protocol(format!("Failed to write ping message - {e}")))?;
        self.logger.verbose("Sent initial ping");

        // Ping test
        for i in 1..self.ping_count + 1 {
            let start_time = Instant::now();
            match protocol::write_message(stream, protocol::PING_MESSAGE) {
                Ok(_) => match stream.read_exact(&mut ping_buffer) {
                    Ok(_) => {
                        self.logger.verbose(&format!("Sent ping {i}"));
                        let ping_time = start_time.elapsed();
                        if ping_buffer == protocol::PING_RESPONSE {
                            successful_pings += 1;
                            self.logger.verbose(&format!("Received ping response {i}"));
                            ping_times.push(ping_time);
                        } else {
                            self.logger
                                .verbose(&format!("Received invalid ping response {i}"));
                        }
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::TimedOut => {
                            self.logger
                                .verbose(&format!("Timed out waiting for ping response {i}"));
                            continue;
                        }
                        ErrorKind::UnexpectedEof
                        | ErrorKind::ConnectionReset
                        | ErrorKind::ConnectionAborted => {
                            self.logger.error(&format!(
                                "Connection error while waiting for ping response {i} - {e}"
                            ));
                            break;
                        }
                        _ => {
                            self.logger
                                .warn(&format!("Failed to read ping response {i} - {e}"));
                        }
                    },
                },
                Err(e) => self
                    .logger
                    .verbose(&format!("Failed to write ping message {i} - {e}")),
            }

            if i < self.ping_count - 1 {
                thread::sleep(Duration::from_millis(100));
            }
        }

        if let Some(mut spinner) = sp {
            spinner.stop_with_message(format!("{msg} ✅ Completed."));
        }

        // Send close message
        protocol::write_message(stream, protocol::PING_DONE).map_err(|e| {
            NetbeatError::protocol(format!("Failed to write ping termination - {e}"))
        })?;
        self.logger.verbose("Sent ping termination message");

        // Report
        let ping_report = PingReport::new(self.ping_count, successful_pings, ping_times);
        if successful_pings > 0 {
            self.logger.info(&format!("{}", ping_report.table_report()));
        } else {
            self.logger
                .error("Ping test failed - no successful responses received");
        }

        Ok(ping_report)
    }

    #[allow(clippy::collapsible_if)]
    fn run_upload_test(
        &self,
        stream: &mut TcpStream,
        buffer: &mut [u8],
        target_bytes: Option<u64>,
        target_time: Duration,
        use_time: bool,
    ) -> Result<SpeedReport> {
        let msg = "🚀 Running upload speed test...";
        let mut sp = if !self.logger.quiet & !self.logger.verbose {
            Some(Spinner::new(Spinners::Dots2, msg.into()))
        } else {
            None
        };
        self.logger.verbose(msg);

        let mut bytes_sent: u64 = 0;
        let update_interval = Duration::from_secs(1);
        let mut iteration_count = 0u64;
        let check_interval = 500;
        let target_bytes = target_bytes.unwrap_or(0);

        // Send initial upload start
        protocol::write_message(stream, protocol::UPLOAD_START).map_err(|e| {
            NetbeatError::protocol(format!("Failed to send upload start message - {e}"))
        })?;

        let start_time = Instant::now();
        let mut last_update = Instant::now();
        // Upload test
        if use_time {
            // Time-based upload test
            while start_time.elapsed() < target_time {
                match protocol::write_message(stream, buffer) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(NetbeatError::protocol(format!(
                            "Failed to send upload buffer - {e}"
                        )));
                    }
                }
                bytes_sent += buffer.len() as u64;
                // iteration_count += 1;
                // if iteration_count % check_interval == 0 {
                //     if last_update.elapsed() >= update_interval {
                //         sp =
                //             reports::print_progress(start_time.elapsed(), bytes_sent, &mut sp, msg);
                //         last_update = Instant::now();
                //     }
                // }
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
                protocol::write_message(stream, &buffer[..to_write as usize]).map_err(|e| {
                    NetbeatError::protocol(format!("Failed to send upload buffer - {e}"))
                })?;
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
        if let Some(mut sp) = sp {
            sp.stop_with_message(format!("{msg} ✅ Completed."));
        }

        // Send close message
        protocol::write_message(stream, protocol::UPLOAD_DONE)
            .map_err(|e| NetbeatError::protocol(format!("Failed to send close message - {e}")))?;

        stream
            .flush()
            .map_err(|e| NetbeatError::protocol(format!("Failed to flush stream - {e}")))?;

        // Report
        let upload_report = SpeedReport::new("upload", upload_time, bytes_sent).unwrap();
        self.logger
            .info(&format!("{}", upload_report.table_report()));
        Ok(upload_report)
    }

    #[allow(clippy::collapsible_if)]
    fn run_download_test(
        &self,
        stream: &mut TcpStream,
        buffer: &mut [u8],
        target_bytes: Option<u64>,
        target_time: Duration,
        use_time: bool,
    ) -> Result<SpeedReport> {
        let msg = "🚀 Running download speed test...";
        let mut sp = if !self.logger.quiet & !self.logger.verbose {
            Some(Spinner::new(Spinners::Dots2, msg.into()))
        } else {
            None
        };
        self.logger.verbose(msg);

        let mut bytes_received: u64 = 0;

        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs(1);
        let mut iteration_count = 0u64;
        let check_interval = 500;
        let target_bytes = target_bytes.unwrap_or(0);

        // Send initial download start
        protocol::write_message(stream, protocol::DOWNLOAD_START).map_err(|e| {
            NetbeatError::protocol(format!("Failed to send download start message - {e}"))
        })?;

        let start_time = Instant::now();
        if use_time {
            // Time-base download test
            while start_time.elapsed() < target_time {
                match stream.read(buffer) {
                    Ok(0) => break,
                    Ok(n) => bytes_received += n as u64,
                    Err(e) => {
                        if let Some(mut sp) = sp {
                            sp.stop();
                        }
                        return Err(NetbeatError::protocol(format!(
                            "Failed to read download buffer - {e}"
                        )));
                    }
                }
                // iteration_count += 1;
                // if iteration_count % check_interval == 0 {
                //     if last_update.elapsed() >= update_interval {
                //         sp = reports::print_progress(
                //             start_time.elapsed(),
                //             bytes_received,
                //             &mut sp,
                //             msg,
                //         );
                //         last_update = Instant::now();
                //     }
                // }
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
                        if let Some(mut sp) = sp {
                            sp.stop();
                        }
                        return Err(NetbeatError::protocol(format!(
                            "Failed to read download buffer - {e}"
                        )));
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
        if let Some(mut sp) = sp {
            sp.stop_with_message(format!("{msg} ✅ Completed."))
        };

        // Send close message
        // protocol::write_message(stream, protocol::DOWNLOAD_DONE)?;
        stream
            .flush()
            .map_err(|e| NetbeatError::protocol(format!("Failed to flush stream - {e}")))?;

        // Report
        let download_report = SpeedReport::new("download", download_time, bytes_received).unwrap();
        self.logger
            .info(&format!("{}", download_report.table_report()));
        Ok(download_report)
    }
}

impl ClientBuilder {
    pub fn new(target: impl Into<String>) -> Self {
        ClientBuilder {
            target: target.into(),
            port: None,
            data: None,
            time: None,
            chunk_size: None,
            ping_count: None,
            return_json: None,
            timeout: None,
            retries: None,
            quiet: None,
            verbose: None,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn data(mut self, data: Option<impl Into<String>>) -> Self {
        self.data = data.map(|d| d.into());
        self
    }

    pub fn time(mut self, time: u64) -> Self {
        self.time = Some(time);
        self
    }

    pub fn chunk_size(mut self, chunk_size: impl Into<String>) -> Result<Self> {
        self.chunk_size = Some(protocol::validate_chunk_size(&chunk_size.into(), "client")?);
        Ok(self)
    }

    pub fn ping_count(mut self, ping_count: u32) -> Self {
        self.ping_count = Some(ping_count);
        self
    }

    pub fn return_json(mut self, return_json: bool) -> Self {
        self.return_json = Some(return_json);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = Some(quiet);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    pub fn build(self) -> Result<Client> {
        Ok(Client {
            socket_addr: SocketAddr::new(
                IpAddr::from_str(&self.target).map_err(|e| {
                    NetbeatError::client(format!("Invalid IP address ({}) - {e}", self.target))
                })?,
                self.port.unwrap_or(config::DEFAULT_PORT),
            ),
            data: match self.data.as_deref() {
                Some(data) => Some(
                    Byte::parse_str(data, false)
                        .map_err(|e| {
                            NetbeatError::client(format!(
                                "Invalid target data ({:?}) - {e}",
                                self.data.unwrap()
                            ))
                        })?
                        .as_u64(),
                ),
                None => None,
            },
            time: self.time.unwrap_or(config::DEFAULT_TEST_DURATION),
            chunk_size: Byte::parse_str(
                self.chunk_size
                    .as_deref()
                    .unwrap_or(config::DEFAULT_CHUNK_SIZE),
                false,
            )
            .map_err(|e| {
                NetbeatError::client(format!("Invalid chunk size ({:?}) - {e}", self.chunk_size))
            })?
            .as_u64(),
            ping_count: self.ping_count.unwrap_or(config::DEFAULT_PING_COUNT),
            return_json: self.return_json.unwrap_or(false),
            timeout: Duration::from_secs(
                self.timeout.unwrap_or(config::DEFAULT_CONNECTION_TIMEOUT),
            ),
            retries: self.retries.unwrap_or(config::DEFAULT_MAX_RETRIES),
            logger: Logger::new(self.verbose.unwrap_or(false), self.quiet.unwrap_or(false)),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_build_client() {
        let client = Client::builder("0.0.0.0")
            .port(8080)
            .data(Some("100MiB"))
            .time(10)
            .chunk_size("1024")
            .unwrap()
            .ping_count(10)
            .return_json(false)
            .timeout(60)
            .retries(5)
            .quiet(true)
            .verbose(false)
            .build()
            .unwrap();

        let _: SocketAddr = client.socket_addr;

        assert_eq!(client.socket_addr.port(), 8080);
        assert_eq!(client.socket_addr.ip().to_string(), "0.0.0.0");
        assert_eq!(client.socket_addr.to_string(), "0.0.0.0:8080");
        assert_eq!(client.data, Some(100 * 1024 * 1024));
        assert_eq!(client.time, 10);
        assert_eq!(client.chunk_size, 1024);
        assert_eq!(client.ping_count, 10);
        assert_eq!(client.return_json, false);
        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.retries, 5);

        let _: Logger = client.logger;
        assert_eq!(client.logger.quiet, true);
        assert_eq!(client.logger.verbose, false);
    }

    #[test]
    fn test_build_client_invalid_input() {
        // Invalid target
        let result = Client::builder("invalid_target").build();

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ClientError { .. }));
            assert!(e.to_string().contains("Invalid IP address"));
        }

        // Invalid Target Data
        let result = Client::builder("0.0.0.0").data(Some("1MMM")).build();

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ClientError { .. }));
            assert!(e.to_string().contains("Invalid target data"));
        }

        // Invalid Chunk Size
        let result = Client::builder("0.0.0.0").chunk_size("1MM");

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ClientError { .. }));
            assert!(e.to_string().contains("Invalid chunk size"));
        }
    }
}
