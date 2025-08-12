//! Network protocol definitions and utilities for netbeat

use rand::RngCore;

/// Ping message sent from client to server
pub const PING_MESSAGE: &[u8] = b"PING";

/// Ping response sent from server to client
pub const PING_RESPONSE: &[u8] = b"PONG";

/// Signal to terminate ping test
pub const PING_TERMINATOR: &[u8] = b"STOP";

/// Signal to indicate upload test completion
pub const UPLOAD_DONE: &[u8] = b"UPLOAD_DONE";

/// Generate a random buffer of specified size for testing
pub fn generate_random_buffer(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut buffer);
    buffer
}
