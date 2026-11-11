use super::behavior::{Behavior, BehaviorInput, JoystickCommand};

#[derive(Debug, Default)]
pub struct EmergencyStop;

impl Behavior for EmergencyStop {
    fn start(&mut self) {}

    fn update(&mut self, _input: BehaviorInput<'_>) -> JoystickCommand {
        JoystickCommand { x: 0, y: 0 }
    }

    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_neutral_joystick_axes() {
        let mut behavior = EmergencyStop;
        let command = behavior.update(BehaviorInput {
            sensor_data: None,
            target_data: None,
        });

        assert_eq!(command.x, 0);
        assert_eq!(command.y, 0);
    }
}
