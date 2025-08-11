use rand::RngCore;

pub const PING_MESSAGE: &[u8] = b"PING";
pub const PING_RESPONSE: &[u8] = b"PONG";
pub const PING_TERMINATOR: &[u8] = b"STOP";

pub fn generate_random_buffer(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut buffer);
    buffer
}
