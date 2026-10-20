#[derive(Debug, Clone)]
pub struct MotionCommand {
    pub linear_velocity: f32,
    pub angular_velocity: f32,
}

pub trait Behavior: Send {
    fn start(&mut self);

    fn update(&mut self) -> MotionCommand;

    fn stop(&mut self);
}