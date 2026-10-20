use tokio::sync::{broadcast, watch};

use crate::behaviors::{
    behavior::{Behavior, BehaviorInput, MotionCommand},
    dummy_go::DummyGo,
    emergency_stop::EmergencyStop,
};
use crate::messages::messages::{BluetoothMessage, ChairStatus, Mode};

pub struct BehaviorOrchestrator {
    current_mode: Mode,
    behavior: Option<Box<dyn Behavior>>,
    sensor_data: Option<Vec<u8>>,
    target_data: Option<Vec<u8>>,
    status: watch::Sender<ChairStatus>,
}

impl BehaviorOrchestrator {
    pub fn new(status: watch::Sender<ChairStatus>) -> Self {
        let mut orchestrator = Self {
            current_mode: Mode::EmergencyStop,
            behavior: None,
            sensor_data: None,
            target_data: None,
            status,
        };
        orchestrator.set_behavior(Box::new(EmergencyStop));
        orchestrator
    }

    pub fn set_behavior(&mut self, behavior: Box<dyn Behavior>) {
        if let Some(mut current_behavior) = self.behavior.replace(behavior) {
            current_behavior.stop();
        }

        if let Some(behavior) = self.behavior.as_mut() {
            behavior.start();
        }
    }

    pub async fn run(mut self, mut receiver: broadcast::Receiver<BluetoothMessage>) {
        loop {
            match receiver.recv().await {
                Ok(BluetoothMessage::ModeChange(mode)) => self.set_mode(mode),
                Ok(BluetoothMessage::SensorData(data)) => {
                    self.sensor_data = Some(data);
                    if let Some(command) = self.update_behavior() {
                        self.handle_motion_command(command);
                    }
                }
                Ok(BluetoothMessage::TargetData(data)) => {
                    self.target_data = Some(data);
                    if let Some(command) = self.update_behavior() {
                        self.handle_motion_command(command);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Behavior orchestrator skipped {skipped} Bluetooth messages");
                }
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    }

    fn update_behavior(&mut self) -> Option<MotionCommand> {
        let input = BehaviorInput {
            sensor_data: self.sensor_data.as_deref(),
            target_data: self.target_data.as_deref(),
        };

        self.behavior
            .as_mut()
            .map(|behavior| behavior.update(input))
    }

    fn handle_motion_command(&self, command: MotionCommand) {
        println!(
            "Motion command: linear={}, angular={}",
            command.linear_velocity, command.angular_velocity
        );
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if self.current_mode == mode {
            return;
        }

        let behavior: Box<dyn Behavior> = match mode {
            Mode::EmergencyStop => Box::new(EmergencyStop),
            Mode::DummyGo => Box::new(DummyGo::default()),
            Mode::GuideFollowing | Mode::AutoDock => {
                eprintln!("Behavior for {mode:?} is not implemented; stopping safely");
                Box::new(EmergencyStop)
            }
        };

        let current_behavior = match mode {
            Mode::EmergencyStop | Mode::GuideFollowing | Mode::AutoDock => "EmergencyStop",
            Mode::DummyGo => "DummyGo",
        };
        self.current_mode = mode;
        self.set_behavior(behavior);
        self.status.send_replace(ChairStatus {
            current_mode: mode,
            current_behavior,
        });
        println!("Mode updated to {:?}", self.current_mode);

        if let Some(command) = self.update_behavior() {
            self.handle_motion_command(command);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_behavior_for_each_mode() {
        let (status, _status_receiver) = watch::channel(ChairStatus {
            current_mode: Mode::EmergencyStop,
            current_behavior: "EmergencyStop",
        });
        let mut orchestrator = BehaviorOrchestrator::new(status.clone());

        orchestrator.set_mode(Mode::EmergencyStop);
        assert_eq!(orchestrator.update_behavior().unwrap().linear_velocity, 0.0);

        orchestrator.set_mode(Mode::DummyGo);
        assert_eq!(status.borrow().current_behavior, "DummyGo");
        assert_eq!(
            orchestrator.update_behavior().unwrap().linear_velocity,
            0.25
        );

        orchestrator.set_mode(Mode::GuideFollowing);
        assert_eq!(status.borrow().current_mode, Mode::GuideFollowing);
        assert_eq!(status.borrow().current_behavior, "EmergencyStop");
        let command = orchestrator.update_behavior().unwrap();
        assert_eq!(command.linear_velocity, 0.0);
        assert_eq!(command.angular_velocity, 0.0);

        orchestrator.set_mode(Mode::AutoDock);
        let command = orchestrator.update_behavior().unwrap();
        assert_eq!(command.linear_velocity, 0.0);
        assert_eq!(command.angular_velocity, 0.0);
    }
}
