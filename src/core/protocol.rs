//! Network protocol definitions and utilities for netbeat

use crate::utils::error::{NetbeatError, Result};
use rand::RngCore;
use std::io::{self, Write};

/// Protocol Version
pub const PROTOCOL_VERSION: &[u8] = b"NETBEAT_1.0";

/// Protocol messages
pub const PING_MESSAGE: &[u8] = b"NETBEAT_PING";
pub const PING_RESPONSE: &[u8] = b"NETBEAT_PONG";
pub const PING_DONE: &[u8] = b"NETBEAT_DONE";
pub const UPLOAD_START: &[u8] = b"NETBEAT_UPLOAD_START";
pub const UPLOAD_DONE: &[u8] = b"NETBEAT_UPLOAD_DONE";
pub const DOWNLOAD_START: &[u8] = b"NETBEAT_DOWNLOAD_START";
pub const DOWNLOAD_DONE: &[u8] = b"NETBEAT_DOWNLOAD_DONE";

/// Simple helper to write message and flush
pub fn write_message(stream: &mut impl Write, message: &[u8]) -> io::Result<()> {
    stream.write_all(message)?;
    stream.flush()
}

/// Generate a random buffer of specified size for testing
pub fn generate_random_buffer(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut buffer);
    buffer
}

/// Helper function to validate chunk size
pub fn validate_chunk_size(s: &str, err_type: &str) -> Result<String> {
    let create_error = |msg: String| {
        if err_type == "server" {
            NetbeatError::server(msg)
        } else {
            NetbeatError::client(msg)
        }
    };

    let byte_size = byte_unit::Byte::parse_str(s, false)
        .map_err(|e| create_error(format!("Invalid chunk size '{s}' - {e}")))?;

    let size = byte_size.as_u64();
    if size < 1024 {
        return Err(create_error("Chunk size must be at least 1KiB".to_string()));
    }
    if size > 1024 * 1024 * 16 {
        // 16MiB max
        return Err(create_error("Chunk size must not exceed 16MiB".to_string()));
    }

    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_buffer_size() {
        let buffer = generate_random_buffer(1024);
        assert_eq!(buffer.len(), 1024);
    }

    #[test]
    fn test_generate_random_buffer_different_calls() {
        let buf1 = generate_random_buffer(100);
        let buf2 = generate_random_buffer(100);
        assert_ne!(buf1, buf2); // Should be different random data
    }

    #[test]
    fn test_protocol_constants() {
        assert_eq!(PROTOCOL_VERSION, b"NETBEAT_1.0");
        assert_eq!(PING_MESSAGE, b"NETBEAT_PING");
        assert_eq!(PING_RESPONSE, b"NETBEAT_PONG");
        assert_eq!(PING_DONE, b"NETBEAT_DONE");
        assert_eq!(UPLOAD_START, b"NETBEAT_UPLOAD_START");
        assert_eq!(UPLOAD_DONE, b"NETBEAT_UPLOAD_DONE");
        assert_eq!(DOWNLOAD_START, b"NETBEAT_DOWNLOAD_START");
        assert_eq!(DOWNLOAD_DONE, b"NETBEAT_DOWNLOAD_DONE");
    }

    #[test]
    fn test_write_message() {
        let mut buffer = Vec::new();
        let message = b"Hello, World!";
        write_message(&mut buffer, message).unwrap();
        assert_eq!(buffer, message);
    }

    #[test]
    fn test_validate_chunk_size() {
        let size = validate_chunk_size("1024", "server").unwrap();
        assert_eq!(size, "1024");

        let size = validate_chunk_size("16MiB", "client").unwrap();
        assert_eq!(size, "16MiB");

        let result = validate_chunk_size("1B", "client");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ClientError { .. }));
            assert!(e.to_string().contains("must be at least 1KiB"));
        }

        let result = validate_chunk_size("1GiB", "server");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ServerError { .. }));
            assert!(e.to_string().contains("must not exceed 16MiB"));
        }

        // Test invalid format
        let result = validate_chunk_size("invalid", "client");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ClientError { .. }));
            assert!(e.to_string().contains("Invalid chunk size"));
        }

        let result = validate_chunk_size("", "server");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, NetbeatError::ServerError { .. }));
            assert!(e.to_string().contains("Invalid chunk size"));
        }
    }
}
