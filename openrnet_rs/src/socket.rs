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
    /// Opens a new R-NET Socket on the specified device.
    ///
    /// The JSM (JoyStick Manager) id should map to the JSM device publishing CAN frames to the chair. Typically the JSM id is 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use openrnet_rs::RnetSock;
    ///
    /// let rsock = RnetSock::new("vcan0", 1).expect("vcan0 is a valid interface!");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an ['RnetSockErr::Create`] if the can device was not found or does not exist on the host machine.
    pub fn new(device: &str, jsm_id: u8) -> Result<Self, RnetSockErr> {
        let sock = CanSocket::open(device).map_err(|_| RnetSockErr::Create)?;
        Ok(RnetSock {
            inner: sock,
            jsm_id,
        })
    }

    /// Opens a new R-NET Socket on can0.
    ///
    /// The JSM (JoyStick Manager) id should map to the JSM device publishing CAN frames to the chair. Typically the JSM id is 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use openrnet_rs::RnetSock;
    ///
    /// let rsock = RnetSock::on_can0(1).expect("can0 is a valid interface!");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an ['RnetSockErr::Create`] if the can device was not found or does not exist on the host machine.
    pub fn on_can0(jsm_id: u8) -> Result<Self, RnetSockErr> {
        RnetSock::new(DEFAULT_CAN_IFACE, jsm_id)
    }

    /// Asynchronously sends a command to the wheelchair.
    ///
    /// 99% of the time [`RnetCommand`] should be passed to the function, but it will also accept raw `CanFrame`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use openrnet_rs::{RnetSock, RnetCommand};
    ///
    /// let rsock = RnetSock::on_can0(1).expect("can0 is a valid interface!");
    /// let _ = rsock.send(RnetCommand::SetSpeed(50)).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`RnetSockErr::Read`] for io errors.
    /// Returns an [`RnetSockErr::CanSocket`] for internal CanSocket errors.
    pub async fn send<F: Into<CanFrame>>(&mut self, command: F) -> Result<(), RnetSockErr> {
        let mut frame = command.into();
        let id = frame.id_word() & !(1 << 31);
        frame.set_id(get_extended_id(id, self.jsm_id).unwrap());
        Ok(self.inner.send(frame.into()).await?)
    }

    /// Asynchronously waits for a CAN Frame with specified id.
    ///
    /// The mask can be used to define bits of the frame that need to match the id and bits that don't. 
    /// For the most part, using `WAIT_MATCH_ID_EXACT`, `WAIT_ACCEPT_ANY_DEVICE`, or `WAIT_MATCH_ANY` will be
    /// sufficient for mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use openrnet_rs::{RnetSock, RnetCommand};
    ///
    /// let rsock = RnetSock::on_can0(1).expect("can0 is a valid interface!");
    /// match rsock
    ///     .wait_for(
    ///         rnet_id::JOYSTICK,
    ///         WAIT_ACCEPT_ANY_DEVICE,
    ///         Some(Duration::from_millis(10)),
    ///     )
    ///     .await
    /// {
    ///     Ok(_) => {
    ///         // spoof the joystick's CAN frame
    ///         rsock
    ///             .send(RnetCommand::Joystick { x: 30, y: 70 })
    ///             .await?;
    ///     }
    ///     Err(RnetSockErr::TimedOut) => { /* No CAN frame with id matching rnet_id::JOYSTICK was found */ }
    ///     Err(e) => { /* Handle the error... */ },
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`RnetSockErr::Read`] for io errors.
    /// Returns an [`RnetSockErr::CanSocket`] for internal CanSocket errors.
    /// Returns an [`RnetSockErr::TimedOut`] if no frame was found during the given timeout.
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

    /// Clears all frames from the receive buffer.
    ///
    /// Before waiting on frames, it is advised to clear the frame buffer so only recent frames are processed.
    ///
    /// # Examples
    ///
    /// ```
    /// use openrnet_rs::RnetSock;
    ///
    /// let rsock = RnetSock::on_can0(1).expect("can0 is a valid interface!");
    /// rsock.clear_recv_buf();
    /// ```
    pub fn clear_recv_buf(&self) {
        while let Ok(_) =  self.inner.try_read_frame() { }
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
