use super::behavior::{Behavior, BehaviorInput, MotionCommand};

#[derive(Debug, Default)]
pub struct EmergencyStop;

impl Behavior for EmergencyStop {
    fn start(&mut self) {}

    fn update(&mut self, _input: BehaviorInput<'_>) -> MotionCommand {
        MotionCommand {
            linear_velocity: 0.0,
            angular_velocity: 0.0,
        }
    }

    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_zero_linear_and_angular_velocity() {
        let mut behavior = EmergencyStop;
        let command = behavior.update(BehaviorInput {
            sensor_data: None,
            target_data: None,
        });

        assert_eq!(command.linear_velocity, 0.0);
        assert_eq!(command.angular_velocity, 0.0);
    }
}
