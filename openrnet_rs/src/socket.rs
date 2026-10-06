use anyhow::Result;
use socketcan::{CanFrame, tokio::CanSocket};

const DEFAULT_CAN_IFACE: &'static str = "can0";

pub struct RnetSock {
    inner: CanSocket,
}

impl RnetSock {
    pub fn new(iface_name: &str) -> Result<Self> {
        Ok(RnetSock {
            inner: CanSocket::open(iface_name)?,
        })
    }

    pub fn default() -> Result<Self> {
        RnetSock::new(DEFAULT_CAN_IFACE)
    }

    pub async fn send<F: Into<CanFrame>>(&self, frame: F) {
        self.send(frame)
    }
}
