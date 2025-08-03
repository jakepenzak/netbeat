use byte_unit::Byte;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetbeatConf {
    pub socket_addr: SocketAddr,
    pub data_size: Option<u64>,
    pub duration: Option<u64>,
    pub chunk_size: u64,
}

impl NetbeatConf {
    pub fn client(
        target: String,
        port: u16,
        data_size: String,
        duration: u64,
        chunk_size: String,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data_size: Some(Byte::parse_str(data_size, false)?.as_u64()),
            duration: Some(duration),
            chunk_size: Byte::parse_str(chunk_size, false)?.as_u64(),
        })
    }

    pub fn server(target: String, port: u16, chunk_size: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data_size: None,
            duration: None,
            chunk_size: Byte::parse_str(chunk_size, false)?.as_u64(),
        })
    }
}
