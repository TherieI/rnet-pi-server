use openrnet::{
    command::{RnetCommand, rnet_id},
    socket::{RnetSock, WAIT_ACCEPT_ANY_DEVICE},
};
use tokio::time::{Duration, Instant};

use crate::error::ChairError;

pub struct Chair {
    sock: RnetSock,
}

impl Chair {
    pub fn new(socket: RnetSock) -> Self {
        Chair { sock: socket }
    }

    async fn move_toward(&mut self, dir: (i8, i8), millis: u64) -> Result<(), ChairError> {
        let limit = Instant::now() + Duration::from_millis(millis);
        while Instant::now() < limit {
            // wait for the JSM's joystick input
            if let Ok(data) = self
                .sock
                .wait_for(
                    rnet_id::JOYSTICK,
                    WAIT_ACCEPT_ANY_DEVICE,
                    Some(Duration::from_millis(9)),
                )
                .await
            {
                if data[0] != 0 || data[1] != 0 {
                    // the user is attempting to gain control of the joystick, halt operations
                    return Err(ChairError::UserInterrupt);
                }
                // and spoof it
                self
                    .sock
                    .send(RnetCommand::Joystick { x: dir.0, y: dir.1 })
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn forward(&mut self, millis: u64) -> Result<(), ChairError> {
        self.move_toward((0, 100), millis).await
    }

    pub async fn set_speed(&mut self, speed: u8) -> Result<(), ChairError> {
        Ok(self.sock.send(RnetCommand::SetSpeed(speed)).await?)
    }
}
