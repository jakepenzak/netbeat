//! Configuration constants and limits for netbeat

use clap::ValueEnum;

/// Default server port
pub const DEFAULT_PORT: u16 = 5050;

/// Default chunk size for data transfer
pub const DEFAULT_CHUNK_SIZE: &str = "64KiB";

/// Default test duration in seconds
pub const DEFAULT_TEST_DURATION: u64 = 10;

// /// Default target data size (defaults to using test duration)
// pub const DEFAULT_TARGET_DATA: Option<String> = None;

/// Default ping count
pub const DEFAULT_PING_COUNT: u32 = 20;

/// Default maximum concurrent connections allowed
pub const DEFAULT_MAX_CONNECTIONS: u32 = 50;

/// Default connection timeout
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;

/// Default number of retries
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default server IP
pub const DEFAULT_BIND_INTERFACE: BindInterface = BindInterface::All;

/// Network interface binding options for the server
#[derive(Debug, Clone, ValueEnum, Copy)]
pub enum BindInterface {
    /// Listen on all network interfaces (0.0.0.0)
    All,
    /// Listen on localhost only (127.0.0.1)
    Localhost,
}

impl BindInterface {
    /// Convert to the actual IP address string
    pub fn to_ip(&self) -> &'static str {
        match self {
            BindInterface::All => "0.0.0.0",
            BindInterface::Localhost => "127.0.0.1",
        }
    }
}

// Add this implementation for clap
impl std::fmt::Display for BindInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindInterface::All => write!(f, "all"),
            BindInterface::Localhost => write!(f, "localhost"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bind_interface_to_ip() {
        assert_eq!(BindInterface::All.to_ip(), "0.0.0.0");
        assert_eq!(BindInterface::Localhost.to_ip(), "127.0.0.1");
    }

    #[test]
    fn test_bind_interface_display() {
        assert_eq!(BindInterface::All.to_string(), "all");
        assert_eq!(BindInterface::Localhost.to_string(), "localhost");
    }
}
