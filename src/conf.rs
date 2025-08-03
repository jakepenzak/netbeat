use byte_unit::Byte;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct NetbeatConf {
    pub socket_addr: SocketAddr,
    pub data_size: Option<u64>,
    pub buffer_size: Option<u64>,
}

impl NetbeatConf {
    pub fn client(target: String, port: u16, data_size: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data_size: Some(Byte::parse_str(data_size, false)?.as_u64()),
            buffer_size: None,
        })
    }

    pub fn server(target: String, port: u16, buffer_size: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            socket_addr: SocketAddr::new(IpAddr::from_str(&target)?, port),
            data_size: None,
            buffer_size: Some(Byte::parse_str(buffer_size, false)?.as_u64()),
        })
    }
}
