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

    pub async fn wait_for(&self, id: u32, timeout: Option<Duration>) -> Result<[u8; 8], RnetSockErr> {
        let fut = async {
            loop {
                let frame = self.inner.read_frame().await?;
                if let CanFrame::Data(f) = frame {
                    // Extended frames have the 31st bit set to 1. Clearing this bit marks an extended frame id
                    // indistinguishable from a standard frame id which might cause issues in the future.
                    if f.id_word() & !(1 << 31) == id {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

use crate::{command::{RnetCommand, rnet_id}, socket::RnetSock};

    #[tokio::test]
    async fn test_wait_for_can() {
        // run `cansend vcan0 02000100#0064` in terminal
        let rsock = RnetSock::new("vcan0").unwrap();
        let res = rsock.wait_for(rnet_id::JOYSTICK | 0x100, Some(Duration::from_secs(10))).await;
        println!("{:?}", res.unwrap());
    }

    #[tokio::test]
    async fn test_send_can() {
        let mut rsock = RnetSock::new("vcan0").unwrap();
        let cmd = RnetCommand::Joystick { x: 80, y: -20 };
        assert!(rsock.send(cmd).await.is_ok());
    }

    #[tokio::test]
    async fn test_recv_then_send() {
        // run `cansend vcan0 02000100#0064` in terminal
        let mut rsock = RnetSock::new("vcan0").unwrap();

        let directions = rsock.wait_for(rnet_id::JOYSTICK | 0x100, Some(Duration::from_secs(10))).await.expect("recved from CAN");

        assert!(rsock.send(RnetCommand::Joystick { x: directions[0] as i8 - 10, y: directions[1] as i8 - 10 }).await.is_ok())
    }
}