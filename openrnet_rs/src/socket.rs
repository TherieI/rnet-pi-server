use std::time::UNIX_EPOCH;

use futures_util::sink::SinkExt;
use socketcan::{CanFrame, EmbeddedFrame, Frame, tokio::CanSocket};
use tokio::time::{Duration, Instant};

use crate::{command::get_extended_id, error::RnetSockErr};

const DEFAULT_CAN_IFACE: &'static str = "can0";
const RNET_WAIT_MAX_TIMEOUT: Duration = Duration::from_secs(100);

pub const WAIT_ACCEPT_ANY_DEVICE: u32 = 0xFFFF_F0FF;
pub const WAIT_MATCH_ID_EXACT: u32 = 0xFFFF_FFFF;
pub const WAIT_MATCH_ANY: u32 = 0x0;

pub struct RnetSock {
    inner: CanSocket,
    jsm_id: u8,
}

impl RnetSock {
    pub fn new(iface_name: &str, jsm_id: u8) -> Result<Self, RnetSockErr> {
        let sock = CanSocket::open(iface_name).map_err(|_| RnetSockErr::Create)?;
        Ok(RnetSock {
            inner: sock,
            jsm_id,
        })
    }

    pub fn on_can0(jsm_id: u8) -> Result<Self, RnetSockErr> {
        RnetSock::new(DEFAULT_CAN_IFACE, jsm_id)
    }

    pub async fn send<F: Into<CanFrame>>(&mut self, frame: F) -> Result<(), RnetSockErr> {
        let mut frame = frame.into();
        let id = frame.id_word() & !(1 << 31);
        frame.set_id(get_extended_id(id, self.jsm_id).unwrap());
        Ok(self.inner.send(frame.into()).await?)
    }

    pub async fn wait_for(
        &self,
        id: u32,
        mask: u32,
        timeout: Option<Duration>,
    ) -> Result<[u8; 8], RnetSockErr> {
        let time_limit = Instant::now() + timeout.unwrap_or(RNET_WAIT_MAX_TIMEOUT);

        while Instant::now() < time_limit {
            let frame = self.inner.read_frame().await?;
            if let CanFrame::Data(f) = frame {
                // Extended frames have the 31st bit set to 1. Clearing this bit marks an extended frame id
                // indistinguishable from a standard frame id which might cause issues in the future.
                let cleared_bit = f.id_word() & !(1 << 31);
                if cleared_bit & mask == id & mask {
                    let mut data = [0u8; 8];
                    let slice = f.data();
                    data[..slice.len()].copy_from_slice(slice);
                    return Ok(data);
                }
            }
        }

        Err(RnetSockErr::TimedOut)
    }

    // Flush all frames from the send buffer
    pub async fn flush(&mut self) -> Result<(), RnetSockErr> {
        self.inner.flush().await?;
        Ok(())
    }

    // Clear all frames from the recv buffer
    pub async fn drain(&self) -> Result<(), RnetSockErr> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        loop {
            let (_, timestamp) = self.inner.read_frame_with_hw_timestamp().await?;
            if timestamp > now {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::SinkExt;
    use socketcan::{EmbeddedFrame, tokio::CanSocket};

    use crate::{
        command::{RnetCommand, rnet_id},
        socket::{RnetSock, WAIT_ACCEPT_ANY_DEVICE},
    };

    fn can_bus_emulator() -> CanSocket {
        CanSocket::open("vcan0").expect("set up vcan0 iface!")
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_wait_for_can() {
        /*
            Ensure the virtual can iface is set up:
            sudo modprobe vcan
            sudo ip link add dev vcan0 type vcan
            sudo ip link set up vcan0
        */
        let mut emulator = can_bus_emulator();
        let rsock = RnetSock::new("vcan0", 1).unwrap();

        let can_emulation = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            emulator
                .send(RnetCommand::Joystick { x: 0, y: 100 }.into())
                .await
                .unwrap()
        };

        let test = async {
            rsock
                .wait_for(
                    rnet_id::JOYSTICK,
                    WAIT_ACCEPT_ANY_DEVICE,
                    Some(Duration::from_secs(10)),
                )
                .await
        };

        let res = tokio::join!(test, can_emulation);
        let data = res.0.unwrap();
        assert_eq!(data[1], 100);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_send_can() {
        /*
            Ensure the virtual can iface is set up:
            sudo modprobe vcan
            sudo ip link add dev vcan0 type vcan
            sudo ip link set up vcan0
        */
        let emulator = can_bus_emulator();
        let mut rsock = RnetSock::new("vcan0", 1).unwrap();

        let can_emulation = async { emulator.read_frame().await };

        let test = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let cmd = RnetCommand::Joystick { x: 80, y: -20 };
            rsock.send(cmd).await
        };

        let res = tokio::join!(test, can_emulation);
        assert!(res.0.is_ok());
        let frame = res.1.unwrap();
        let data = frame.data();
        assert_eq!(&[(80i8) as u8, (-20i8) as u8], data);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_recv_then_send() {
        /*
            Ensure the virtual can iface is set up:
            sudo modprobe vcan
            sudo ip link add dev vcan0 type vcan
            sudo ip link set up vcan0
        */
        let mut emulator = can_bus_emulator();
        let mut rsock = RnetSock::new("vcan0", 1).unwrap();

        let can_emulation = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            emulator
                .send(RnetCommand::Joystick { x: 0, y: 100 }.into())
                .await
                .unwrap();
            emulator.read_frame().await
        };

        let test = async {
            let data = rsock
                .wait_for(
                    rnet_id::JOYSTICK,
                    WAIT_ACCEPT_ANY_DEVICE,
                    Some(Duration::from_secs(10)),
                )
                .await
                .unwrap();
            rsock
                .send(RnetCommand::Joystick {
                    x: data[0] as i8 - 10,
                    y: data[1] as i8 - 10,
                })
                .await
        };

        let res = tokio::join!(test, can_emulation);
        assert!(res.0.is_ok());
        let frame = res.1.unwrap();
        let data = frame.data();
        assert_eq!(&[(-10i8) as u8, (90i8) as u8], data);
    }
}
