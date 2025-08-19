use super::{config, protocol};
use crate::utils::logging::Logger;
use anyhow::{Context, Result};
use byte_unit::Byte;
use spinners::{Spinner, Spinners};
use std::{
    io::{self, Read},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Server {
    pub socket_addr: SocketAddr,
    pub chunk_size: u64,
    pub max_connections: u32,
    pub logger: Logger,
}

#[derive(Debug, Default)]
pub struct ServerBuilder {
    interface: Option<config::BindInterface>,
    port: Option<u16>,
    chunk_size: Option<String>,
    max_connections: Option<u32>,
    quiet: Option<bool>,
    verbose: Option<bool>,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    pub fn listen(&self) -> Result<()> {
        let listener = TcpListener::bind(self.socket_addr)?;
        self.logger.info(&format!(
            "🌐 Server Listening on {}",
            listener.local_addr()?
        ));

        let connection_count = Arc::new(Mutex::new(0usize));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    {
                        let mut count = connection_count.lock().unwrap();
                        if *count >= self.max_connections as usize {
                            self.logger.error(&format!(
                                "Maximum connections reached, rejecting {}.",
                                stream.peer_addr()?
                            ));
                            drop(stream);
                            continue;
                        }
                        *count += 1;
                    }
                    stream.set_nodelay(true)?;
                    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
                    self.logger
                        .info(&format!("\n🌐 New connection from {}", stream.peer_addr()?));

                    let count_clone = Arc::clone(&connection_count);
                    let chunk_size = self.chunk_size;
                    let logger = self.logger.clone();
                    thread::spawn(move || {
                        let result = handle_client(stream, chunk_size, &logger);
                        if let Err(e) = result {
                            logger.error(&format!("Error handling client: {e}"));
                        }
                        let mut count = count_clone.lock().unwrap();
                        *count -= 1;
                    });
                }
                Err(e) => self.logger.error(&format!("Connection failed: {e}")),
            }
        }
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, chunk_size: u64, logger: &Logger) -> Result<()> {
    // Ping Test
    handle_ping_test(&mut stream, logger)?;

    thread::sleep(Duration::from_millis(50));

    // Upload Test
    handle_upload_test(&mut stream, chunk_size, logger)?;

    thread::sleep(Duration::from_millis(50));

    // Download Test
    handle_download_test(&mut stream, chunk_size, logger)?;

    Ok(())
}

fn handle_ping_test(stream: &mut TcpStream, logger: &Logger) -> Result<()> {
    let msg = "🏓 Running ping test for client...";
    let sp = if !logger.quiet & !logger.verbose {
        Some(Spinner::new(Spinners::Dots2, msg.into()))
    } else {
        None
    };
    logger.verbose(msg);

    stream.set_read_timeout(Some(Duration::from_secs(30)))?;

    let mut ping_buffer = [0u8; protocol::PING_MESSAGE.len()];
    let mut ping_count = 0;

    loop {
        match stream.read_exact(&mut ping_buffer) {
            Ok(_) => {
                if ping_buffer == protocol::PING_DONE {
                    logger.verbose(&format!("Ping test completed after {ping_count} pings"));
                    break;
                } else if ping_buffer == protocol::PING_MESSAGE {
                    protocol::write_message(stream, protocol::PING_RESPONSE)
                        .context("Failed to send ping response")?;
                    ping_count += 1;
                    logger.verbose(&format!("Ping response sent on ping number {ping_count}"));
                } else {
                    logger.warn(&format!(
                        "Received unexpected message during ping test: {:?}",
                        ping_buffer
                    ));
                    continue;
                }
            }
            Err(e) => {
                logger.error(&format!("Error reading from client: {e}"));
                break;
            }
        }
    }
    if let Some(mut sp) = sp {
        sp.stop_with_message(format!("{msg} ✅ Completed."));
    }
    Ok(())
}

fn handle_upload_test(stream: &mut TcpStream, chunk_size: u64, logger: &Logger) -> Result<()> {
    let mut buffer = vec![0u8; chunk_size as usize];
    let msg = "🚀 Running upload speed test for client...";
    let sp = if !logger.quiet & !logger.verbose {
        Some(Spinner::new(Spinners::Dots2, msg.into()))
    } else {
        None
    };
    logger.verbose(msg);

    // Wait for upload signal
    let mut start_buf = [0u8; protocol::UPLOAD_START.len()];
    stream.read_exact(&mut start_buf)?;
    if start_buf != *protocol::UPLOAD_START {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected upload start").into());
    }

    // Read data until termination signal
    let mut done_buffer = Vec::new();
    let done_marker = protocol::UPLOAD_DONE;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                done_buffer.extend_from_slice(&buffer[..n]);
                if done_buffer.len() >= done_marker.len() {
                    let start_pos = done_buffer.len() - done_marker.len();
                    if &done_buffer[start_pos..] == done_marker {
                        break;
                    }
                }
            }
            Err(e) => {
                logger.error(&format!("Error reading from client: {e}"));
                break;
            }
        }
    }
    if let Some(mut sp) = sp {
        sp.stop_with_message(format!("{msg} ✅ Completed."));
    }
    Ok(())
}

fn handle_download_test(stream: &mut TcpStream, chunk_size: u64, logger: &Logger) -> Result<()> {
    let random_buffer = protocol::generate_random_buffer(chunk_size as usize);

    let msg = "🚀 Running download speed test for client...";
    let sp = if !logger.quiet & !logger.verbose {
        Some(Spinner::new(Spinners::Dots2, msg.into()))
    } else {
        None
    };
    logger.verbose(msg);

    // Wait for download signal
    let mut start_buf = [0u8; protocol::DOWNLOAD_START.len()];
    stream.read_exact(&mut start_buf)?;
    if start_buf != *protocol::DOWNLOAD_START {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected download start").into());
    }

    stream.set_write_timeout(Some(Duration::from_secs(1)))?;

    loop {
        match protocol::write_message(stream, &random_buffer) {
            Ok(_) => {}
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::ConnectionReset => {
                    // Assuming client finished - this is brittle logic
                    break;
                }
                _ => {
                    logger.error(&format!("Unexpected error in download test: {e}"));
                    break;
                }
            },
        }
    }
    if let Some(mut sp) = sp {
        sp.stop_with_message(format!("{msg} ✅ Completed."));
    }
    Ok(())
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interface(mut self, interface: config::BindInterface) -> Self {
        self.interface = Some(interface);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn chunk_size(mut self, chunk_size: impl Into<String>) -> Self {
        self.chunk_size = Some(chunk_size.into());
        self
    }

    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = Some(max_connections);
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

    pub fn build(self) -> Result<Server> {
        Ok(Server {
            socket_addr: SocketAddr::new(
                IpAddr::from_str(
                    self.interface
                        .unwrap_or(config::DEFAULT_BIND_INTERFACE)
                        .to_ip(),
                )?,
                self.port.unwrap_or(config::DEFAULT_PORT),
            ),
            chunk_size: Byte::parse_str(
                self.chunk_size
                    .unwrap_or(config::DEFAULT_CHUNK_SIZE.to_string()),
                false,
            )?
            .as_u64(),
            max_connections: self
                .max_connections
                .unwrap_or(config::DEFAULT_MAX_CONNECTIONS),
            logger: Logger::new(self.verbose.unwrap_or(false), self.quiet.unwrap_or(false)),
        })
    }
}
