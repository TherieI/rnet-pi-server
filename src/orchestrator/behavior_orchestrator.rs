use tokio::sync::broadcast;

use crate::messages::messages::{BluetoothMessage, Mode};

pub struct BehaviorOrchestrator {
    current_mode: Mode,
}

impl BehaviorOrchestrator {
    pub fn new() -> Self {
        Self {
            current_mode: Mode::EmergencyStop,
        }
    }

    pub async fn run(mut self, mut receiver: broadcast::Receiver<BluetoothMessage>) {
        loop {
            match receiver.recv().await {
                Ok(BluetoothMessage::ModeChange(mode)) => self.set_mode(mode),
                Ok(BluetoothMessage::TargetData(data)) => {
                    println!("Target data received: {} bytes", data.len());
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Behavior orchestrator skipped {skipped} Bluetooth messages");
                }
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
        println!("Mode updated to {:?}", self.current_mode);
    }
}
