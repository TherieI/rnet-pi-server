use tokio::sync::{broadcast, watch};

use crate::behaviors::{
    behavior::{Behavior, BehaviorInput, JoystickCommand},
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
    joystick_commands: watch::Sender<JoystickCommand>,
}

impl BehaviorOrchestrator {
    pub fn new(
        status: watch::Sender<ChairStatus>,
        joystick_commands: watch::Sender<JoystickCommand>,
    ) -> Self {
        let mut orchestrator = Self {
            current_mode: Mode::EmergencyStop,
            behavior: None,
            sensor_data: None,
            target_data: None,
            status,
            joystick_commands,
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
            let deadline = self
                .behavior
                .as_ref()
                .and_then(|behavior| behavior.next_update_deadline());
            tokio::select! {
                message = receiver.recv() => {
                    match message {
                        Ok(BluetoothMessage::ModeChange(mode)) => self.set_mode(mode),
                        Ok(BluetoothMessage::SensorData(data)) => {
                            self.sensor_data = Some(data);
                            if let Some(command) = self.update_behavior() {
                                self.handle_joystick_command(command);
                            }
                        }
                        Ok(BluetoothMessage::TargetData(data)) => {
                            self.target_data = Some(data);
                            if let Some(command) = self.update_behavior() {
                                self.handle_joystick_command(command);
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            eprintln!("Behavior orchestrator skipped {skipped} Bluetooth messages");
                        }
                        Err(broadcast::error::RecvError::Closed) => return,
                    }
                }
                _ = wait_until(deadline) => {
                    if let Some(command) = self.update_behavior() {
                        self.handle_joystick_command(command);
                    }
                }
            }
        }
    }

    fn update_behavior(&mut self) -> Option<JoystickCommand> {
        let input = BehaviorInput {
            sensor_data: self.sensor_data.as_deref(),
            target_data: self.target_data.as_deref(),
        };

        self.behavior
            .as_mut()
            .map(|behavior| behavior.update(input))
    }

    fn handle_joystick_command(&self, command: JoystickCommand) {
        println!("Joystick command: x={}, y={}", command.x, command.y);
        if self.joystick_commands.send(command).is_err() {
            eprintln!("Chair controller is unavailable; joystick command was not delivered");
        }
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
            self.handle_joystick_command(command);
        }
    }
}

async fn wait_until(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
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
        let (joystick_commands, joystick_receiver) = watch::channel(JoystickCommand { x: 0, y: 0 });
        let mut orchestrator = BehaviorOrchestrator::new(status.clone(), joystick_commands);

        orchestrator.set_mode(Mode::EmergencyStop);
        assert_eq!(orchestrator.update_behavior().unwrap().y, 0);
        assert_eq!(joystick_receiver.borrow().y, 0);

        orchestrator.set_mode(Mode::DummyGo);
        assert_eq!(status.borrow().current_behavior, "DummyGo");
        assert_eq!(joystick_receiver.borrow().y, 25);
        assert_eq!(orchestrator.update_behavior().unwrap().y, 25);

        orchestrator.set_mode(Mode::GuideFollowing);
        assert_eq!(status.borrow().current_mode, Mode::GuideFollowing);
        assert_eq!(status.borrow().current_behavior, "EmergencyStop");
        let command = orchestrator.update_behavior().unwrap();
        assert_eq!(command.x, 0);
        assert_eq!(command.y, 0);

        orchestrator.set_mode(Mode::AutoDock);
        let command = orchestrator.update_behavior().unwrap();
        assert_eq!(command.x, 0);
        assert_eq!(command.y, 0);
    }

    #[tokio::test]
    async fn dummy_go_stops_automatically_after_its_duration() {
        let (status, _status_receiver) = watch::channel(ChairStatus {
            current_mode: Mode::EmergencyStop,
            current_behavior: "EmergencyStop",
        });
        let (joystick_commands, mut joystick_receiver) =
            watch::channel(JoystickCommand { x: 0, y: 0 });
        let (messages, receiver) = broadcast::channel(4);
        let orchestrator = BehaviorOrchestrator::new(status, joystick_commands);
        let task = tokio::spawn(orchestrator.run(receiver));

        messages
            .send(BluetoothMessage::ModeChange(Mode::DummyGo))
            .unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if joystick_receiver.borrow_and_update().y == 25 {
                    break;
                }
                joystick_receiver.changed().await.unwrap();
            }
        })
        .await
        .expect("DummyGo did not start moving");

        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            loop {
                if joystick_receiver.borrow_and_update().y == 0 {
                    break;
                }
                joystick_receiver.changed().await.unwrap();
            }
        })
        .await
        .expect("DummyGo did not stop automatically");

        drop(messages);
        task.await.unwrap();
    }
}
