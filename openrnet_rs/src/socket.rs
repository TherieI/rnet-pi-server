use anyhow::Result;
use socketcan::{CanFrame, EmbeddedFrame, ExtendedId, Frame, tokio::CanSocket};

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

    pub async fn wait_for(&self, id: u32, mask: u32, timeout: Option<Duration>) -> Result<[u8; 8], WaitError> {
        let fut = async {
            loop {
                let frame = self.inner.read_frame().await.map_err(WaitError::Socket)?;
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
            Some(d) => tokio::time::timeout(d, fut).await.map_err(|_| WaitError::TimedOut)?,
            None => fut.await,
        }
    }
}
