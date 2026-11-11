#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoystickCommand {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, Copy)]
pub struct BehaviorInput<'a> {
    pub sensor_data: Option<&'a [u8]>,
    pub target_data: Option<&'a [u8]>,
}

pub trait Behavior: Send {
    fn start(&mut self);

    fn update(&mut self, input: BehaviorInput<'_>) -> JoystickCommand;

    fn stop(&mut self);

    fn next_update_deadline(&self) -> Option<tokio::time::Instant> {
        None
    }
}
