use crate::core::config::{self, BindInterface};
use clap::Args;

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Target server IP address or hostname
    pub target: String,
    /// Target port on server (1-65535)
    #[arg(short, long, default_value_t = config::DEFAULT_PORT, value_parser = clap::value_parser!(u16).range(1..=65535))]
    pub port: u16,
    /// Time limit per test direction in seconds (1-3600).
    #[arg(short, long, default_value_t = config::DEFAULT_TEST_DURATION, value_parser = clap::value_parser!(u64).range(1..=3600))]
    pub time: u64,
    /// Target size of data to be uploaded/downloaded in the speed test including units (eg, 100KiB, 10MiB, 1GiB). Instead of time.
    #[arg(short, long, default_value = config::DEFAULT_TARGET_DATA, value_parser = validate_data_size)]
    pub data: String,
    /// Buffer size for read/write operations (eg, 32KiB, 64KiB, 128KiB).
    #[arg(short, long, default_value = config::DEFAULT_CHUNK_SIZE, value_parser = validate_chunk_size)]
    pub chunk_size: String,
    /// Number of pings to perform for ping test (1-1000)
    #[arg(long, default_value_t = config::DEFAULT_PING_COUNT, value_parser = clap::value_parser!(u32).range(1..=1000))]
    pub ping_count: u32,
    /// Return results as json to stdout
    #[arg(short, long)]
    pub json: bool,
    /// Connection timeout in seconds
    #[arg(long, default_value_t = config::DEFAULT_CONNECTION_TIMEOUT)]
    pub timeout: u64,
    /// Number of retry attempts on connection failure
    #[arg(long, default_value_t = config::DEFAULT_MAX_RETRIES)]
    pub retries: u32,
    /// Suppress progress output (results & errors only)
    #[arg(short, long)]
    pub quiet: bool,
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Network interface to bind server to: 'all' (0.0.0.0) or 'localhost' (127.0.0.1)
    #[arg(short, long, default_value_t = config::DEFAULT_BIND_INTERFACE, hide_possible_values = true)]
    pub interface: BindInterface,
    /// Port to listen on (1-65535)
    #[arg(short, long, default_value_t = config::DEFAULT_PORT, value_parser = clap::value_parser!(u16).range(1..=65535))]
    pub port: u16,
    /// Buffer size for data transfer (eg, 32KiB, 64KiB, 128KiB).
    #[arg(short, long, default_value = config::DEFAULT_CHUNK_SIZE, value_parser=validate_chunk_size)]
    pub chunk_size: String,
    /// Maximum concurrent connections
    #[arg(long, default_value_t = config::DEFAULT_MAX_CONNECTIONS)]
    pub connections: u32,
    /// Suppress all output (errors only)
    #[arg(short, long)]
    pub quiet: bool,
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

// Validation functions
fn validate_data_size(s: &str) -> Result<String, String> {
    if s == "0" {
        return Ok(s.to_string());
    }

    byte_unit::Byte::parse_str(s, false)
        .map(|_| s.to_string())
        .map_err(|e| format!("Invalid data size '{s}': {e}"))
}

fn validate_chunk_size(s: &str) -> Result<String, String> {
    let byte_size = byte_unit::Byte::parse_str(s, false)
        .map_err(|e| format!("Invalid chunk size '{s}': {e}"))?;

    let size = byte_size.as_u64();
    if size < 1024 {
        return Err("Chunk size must be at least 1KiB".to_string());
    }
    if size > 1024 * 1024 * 16 {
        // 16MiB max
        return Err("Chunk size must not exceed 16MiB".to_string());
    }

    Ok(s.to_string())
}
