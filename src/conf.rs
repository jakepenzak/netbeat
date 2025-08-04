use byte_unit::Byte;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetbeatConf {
    pub socket_addr: SocketAddr,
    pub data: Option<u64>,
    pub time: Option<u64>,
    pub chunk_size: u64,
}

impl NetbeatConf {
    pub fn client(
        target: String,
        port: u16,
        data: String,
        time: u64,
        chunk_size: String,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data: Some(Byte::parse_str(data, false)?.as_u64()),
            time: Some(time),
            chunk_size: Byte::parse_str(chunk_size, false)?.as_u64(),
        })
    }

    pub fn server(target: String, port: u16, chunk_size: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data: None,
            time: None,
            chunk_size: Byte::parse_str(chunk_size, false)?.as_u64(),
        })
    }
}
