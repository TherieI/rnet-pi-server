use std::time::Duration;

use tokio::sync::watch;

use super::{chair::Chair, error::ChairError, mock_chair::MockChair};
use crate::behaviors::behavior::MotionCommand;

const COMMAND_INTERVAL: Duration = Duration::from_millis(100);

enum ChairDevice {
    Real(Chair),
    Mock(MockChair),
}

impl ChairDevice {
    async fn move_in_direction(
        &mut self,
        direction: (i8, i8),
        millis: u64,
    ) -> Result<(), ChairError> {
        match self {
            Self::Real(chair) => chair.move_in_direction(direction, millis).await,
            Self::Mock(chair) => chair.move_in_direction(direction, millis).await,
        }
    }
}

pub struct ChairController {
    chair: ChairDevice,
}

impl ChairController {
    pub fn new() -> Result<Self, ChairError> {
        let socket = openrnet::socket::RnetSock::on_can0(0)?;
        Ok(Self {
            chair: ChairDevice::Real(Chair::new(socket)),
        })
    }

    pub fn new_mock() -> (Self, watch::Receiver<Option<(i8, i8)>>) {
        let (chair, observed_direction) = MockChair::new();
        (
            Self {
                chair: ChairDevice::Mock(chair),
            },
            observed_direction,
        )
    }

    pub async fn run(
        mut self,
        mut commands: watch::Receiver<MotionCommand>,
    ) -> Result<(), ChairError> {
        let mut command = commands.borrow_and_update().clone();
        let mut interval = tokio::time::interval(COMMAND_INTERVAL);

        loop {
            tokio::select! {
                changed = commands.changed() => {
                    if changed.is_err() {
                        return Ok(());
                    }
                    command = commands.borrow_and_update().clone();
                }
                _ = interval.tick() => {
                    let direction = Self::command_to_direction(&command);
                    match self.chair.move_in_direction(direction, COMMAND_INTERVAL.as_millis() as u64).await {
                        Ok(()) => {}
                        Err(ChairError::UserInterrupt) => {
                            eprintln!("Chair control interrupted by user joystick input");
                            command = MotionCommand {
                                linear_velocity: 0.0,
                                angular_velocity: 0.0,
                            };
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        }
    }

    fn command_to_direction(command: &MotionCommand) -> (i8, i8) {
        (
            Self::velocity_to_axis(command.angular_velocity),
            Self::velocity_to_axis(command.linear_velocity),
        )
    }

    fn velocity_to_axis(velocity: f32) -> i8 {
        (velocity.clamp(-1.0, 1.0) * 100.0).round() as i8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messages::messages::{BluetoothMessage, ChairStatus, Mode},
        orchestrator::behavior_orchestrator::BehaviorOrchestrator,
    };

    #[test]
    fn maps_linear_and_angular_velocity_to_joystick_axes() {
        let direction = ChairController::command_to_direction(&MotionCommand {
            linear_velocity: 0.5,
            angular_velocity: -0.25,
        });

        assert_eq!(direction, (-25, 50));
    }

    #[test]
    fn clamps_motion_to_the_joystick_range() {
        let direction = ChairController::command_to_direction(&MotionCommand {
            linear_velocity: 2.0,
            angular_velocity: -2.0,
        });

        assert_eq!(direction, (-100, 100));
    }

    #[tokio::test]
    async fn orchestrator_commands_reach_the_mock_chair() {
        let (controller, mut observed_direction) = ChairController::new_mock();
        let (motion_tx, motion_rx) = watch::channel(MotionCommand {
            linear_velocity: 0.0,
            angular_velocity: 0.0,
        });
        let (status_tx, _) = watch::channel(ChairStatus {
            current_mode: Mode::EmergencyStop,
            current_behavior: "EmergencyStop",
        });
        let (messages_tx, messages_rx) = tokio::sync::broadcast::channel(4);
        let orchestrator_task =
            tokio::spawn(BehaviorOrchestrator::new(status_tx, motion_tx).run(messages_rx));
        let controller_task = tokio::spawn(controller.run(motion_rx));

        messages_tx
            .send(BluetoothMessage::ModeChange(Mode::DummyGo))
            .unwrap();

        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if *observed_direction.borrow_and_update() == Some((0, 25)) {
                    break;
                }
                observed_direction.changed().await.unwrap();
            }
        })
        .await
        .expect("mock chair did not receive the orchestrator motion command");

        drop(messages_tx);
        orchestrator_task.await.unwrap();
        controller_task.await.unwrap().unwrap();
    }
}
