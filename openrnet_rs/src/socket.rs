use socketcan::{CanFrame, EmbeddedFrame, Frame, tokio::CanSocket};
use tokio::time::Duration;
use futures_util::sink::SinkExt;

use crate::error::RnetSockErr;


const DEFAULT_CAN_IFACE: &'static str = "can0";

pub struct RnetSock {
    inner: CanSocket,
}

impl RnetSock {
    pub fn new(iface_name: &str) -> Result<Self, RnetSockErr> {
        Ok(RnetSock {
            inner: CanSocket::open(iface_name).map_err(|_| RnetSockErr::Create)?,
        })
    }

    pub fn default() -> Result<Self, RnetSockErr> {
        RnetSock::new(DEFAULT_CAN_IFACE)
    }

    pub async fn send<F: Into<CanFrame>>(&mut self, frame: F) -> Result<(), RnetSockErr>{
        Ok(self.inner.send(frame.into()).await?)
    }

    pub async fn wait_for(&self, id: u32, mask: u32, timeout: Option<Duration>) -> Result<[u8; 8], RnetSockErr> {
        let fut = async {
            loop {
                let frame = self.inner.read_frame().await?;
                if let CanFrame::Data(f) = frame {
                    if f.id_word() & mask == id & mask {
                        let mut data = [0u8; 8];
                        let slice = f.data();
                        data[..slice.len()].copy_from_slice(slice);
                        return Ok(data);
                    }
                }
            }
        };
        match timeout {
            Some(d) => tokio::time::timeout(d, fut).await.map_err(|_| RnetSockErr::TimedOut)?,
            None => fut.await,
        }
    }
}
