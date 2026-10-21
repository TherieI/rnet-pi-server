use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use openrnet::{
    command::{RnetCommand, rnet_id},
    error::RnetSockErr,
    socket::{RnetSock, WAIT_ACCEPT_ANY_DEVICE},
};
use tokio::time::{Duration, Instant};

use crate::error::ChairError;

pub struct Chair {
    sock: RnetSock,
    locked: Arc<AtomicBool>
}

impl Chair {
    pub fn new(socket: RnetSock) -> Self {
        Chair { sock: socket, locked: Arc::new(false.into()) }
    }

    pub async fn move_in_direction(
        &mut self,
        dir: (i8, i8),
        millis: u64,
    ) -> Result<(), ChairError> {
        let limit = Instant::now() + Duration::from_millis(millis);

        self.sock.clear_recv_buf();
        while !self.locked.load(Ordering::Relaxed) && Instant::now() < limit {
            // wait for the JSM's joystick input
            match self
                .sock
                .wait_for(
                    rnet_id::JOYSTICK,
                    WAIT_ACCEPT_ANY_DEVICE,
                    Some(Duration::from_millis(9)),
                )
                .await
            {
                Ok(data) => {
                    if data[0] != 0 || data[1] != 0 {
                        // the user is attempting to gain control of the joystick, halt operations
                        return Err(ChairError::UserInterrupt);
                    }
                    // spoof CAN frame
                    self.sock
                        .send(RnetCommand::Joystick { x: dir.0, y: dir.1 })
                        .await?;
                }
                Err(RnetSockErr::TimedOut) => { /* timing out is perfectly fine, continue the loop */
                }
                Err(e) => return Err(e.into()),
            }
        }

        if self.locked.load(Ordering::Relaxed) {
            // `Chair::stop` called
            Err(ChairError::EmergencyStop)
        } else {
            Ok(())
        }
    }

    pub async fn forward(&mut self, millis: u64) -> Result<(), ChairError> {
        self.move_in_direction((0, 100), millis).await
    }

    pub async fn backward(&mut self, millis: u64) -> Result<(), ChairError> {
        self.move_in_direction((0, -100), millis).await
    }

    pub async fn left(&mut self, millis: u64) -> Result<(), ChairError> {
        self.move_in_direction((-100, 0), millis).await
    }

    pub async fn right(&mut self, millis: u64) -> Result<(), ChairError> {
        self.move_in_direction((100, 0), millis).await
    }

    pub async fn set_speed(&mut self, speed: u8) -> Result<(), ChairError> {
        Ok(self.sock.send(RnetCommand::SetSpeed(speed)).await?)
    }

    pub async fn honk(&mut self, millis: u64) -> Result<(), ChairError> {
        self.sock.send(RnetCommand::HornStart).await?;
        tokio::time::sleep(Duration::from_millis(millis)).await;
        self.sock.send(RnetCommand::HornStop).await?;
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.locked.store(true, Ordering::Relaxed);
    }

    pub async fn unlock(&mut self) {
        self.locked.store(false, Ordering::Relaxed);
    }
}
