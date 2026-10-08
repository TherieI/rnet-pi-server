pub mod command;
pub mod error;
pub mod socket;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        command::{RnetCommand, rnet_id},
        socket::{RnetSock, WAIT_ACCEPT_ANY_DEVICE},
    };

    #[tokio::test]
    async fn move_wheelchair_forward() {
        let num_secs = 1;

        let mut rsock = RnetSock::new("can0", 0).unwrap();
        let forward = RnetCommand::Joystick { x: 0, y: 100 };

        let fut = async {
            loop {
                // wait for the JSM's joystick input
                if rsock
                    .wait_for(
                        rnet_id::JOYSTICK,
                        WAIT_ACCEPT_ANY_DEVICE,
                        Some(Duration::from_millis(9)),
                    )
                    .await
                    .is_ok()
                {
                    // and spoof it
                    let _ = rsock.send(forward).await;
                }
            }
        };

        let _ = tokio::time::timeout(Duration::from_secs(num_secs), fut).await;
    }
}
