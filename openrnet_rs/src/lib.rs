pub mod command;
pub mod socket;
pub mod error;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame, Frame};

    use crate::{command::{RnetCommand, rnet_id}, socket::RnetSock};

    use super::*;

    #[tokio::test]
    async fn move_wheelchair_forward() {
        let num_secs = 3;

        let mut rsock = RnetSock::on_can0().unwrap();
        let forward = RnetCommand::Joystick { x: 0, y: 100 };

        let fut = async {
            loop {
                // wait for the JSM's joystick input
                let _ = rsock.wait_for(rnet_id::JOYSTICK | 0x100, Some(Duration::from_millis(9))).await;
                // and spoof it
                let _ = rsock.send(forward).await;
            }
        };

        tokio::time::timeout(Duration::from_secs(num_secs), fut).await;
    }
}
