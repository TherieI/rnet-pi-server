use std::time::Duration;

use super::behavior::{Behavior, BehaviorInput, MotionCommand};

const FORWARD_VELOCITY: f32 = 0.25;
const FORWARD_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Default)]
pub struct DummyGo {
    started_at: Option<tokio::time::Instant>,
    finished: bool,
}

impl Behavior for DummyGo {
    fn start(&mut self) {
        self.started_at = Some(tokio::time::Instant::now());
        self.finished = false;
    }

    fn update(&mut self, _input: BehaviorInput<'_>) -> MotionCommand {
        let is_running = self
            .started_at
            .is_some_and(|started_at| started_at.elapsed() < FORWARD_DURATION);
        if !is_running {
            self.finished = true;
        }

        MotionCommand {
            linear_velocity: if is_running { FORWARD_VELOCITY } else { 0.0 },
            angular_velocity: 0.0,
        }
    }

    fn stop(&mut self) {
        self.started_at = None;
        self.finished = true;
    }

    fn next_update_deadline(&self) -> Option<tokio::time::Instant> {
        if self.finished {
            None
        } else {
            self.started_at
                .map(|started_at| started_at + FORWARD_DURATION)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_forward_motion_for_a_limited_time() {
        let mut behavior = DummyGo::default();
        let input = BehaviorInput {
            sensor_data: None,
            target_data: None,
        };

        behavior.start();
        let command = behavior.update(input);
        assert_eq!(command.linear_velocity, FORWARD_VELOCITY);
        assert_eq!(command.angular_velocity, 0.0);

        behavior.started_at = Some(tokio::time::Instant::now() - FORWARD_DURATION);
        let command = behavior.update(input);
        assert_eq!(command.linear_velocity, 0.0);
        assert_eq!(command.angular_velocity, 0.0);
    }

    #[test]
    fn commands_stop_after_stop_is_called() {
        let mut behavior = DummyGo::default();
        let input = BehaviorInput {
            sensor_data: None,
            target_data: None,
        };

        behavior.start();
        behavior.stop();

        let command = behavior.update(input);
        assert_eq!(command.linear_velocity, 0.0);
        assert_eq!(command.angular_velocity, 0.0);
    }
}
