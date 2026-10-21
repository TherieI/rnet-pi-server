#[derive(Debug, Clone)]
pub struct MotionCommand {
    pub linear_velocity: f32,
    pub angular_velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct BehaviorInput<'a> {
    pub sensor_data: Option<&'a [u8]>,
    pub target_data: Option<&'a [u8]>,
}

pub trait Behavior: Send {
    fn start(&mut self);

    fn update(&mut self, input: BehaviorInput<'_>) -> MotionCommand;

    fn stop(&mut self);

    fn next_update_deadline(&self) -> Option<tokio::time::Instant> {
        None
    }
}
