use tokio::sync::watch;

pub struct MockChair {
    last_direction: Option<(i8, i8)>,
    direction: watch::Sender<Option<(i8, i8)>>,
}

impl MockChair {
    pub fn new() -> (Self, watch::Receiver<Option<(i8, i8)>>) {
        let (direction, receiver) = watch::channel(None);
        (
            Self {
                last_direction: None,
                direction,
            },
            receiver,
        )
    }

    pub async fn move_in_direction(
        &mut self,
        direction: (i8, i8),
        _millis: u64,
    ) -> Result<(), super::error::ChairError> {
        if self.last_direction != Some(direction) {
            println!(
                "Mock chair motion: angular={}, linear={}",
                direction.0, direction.1
            );
            self.last_direction = Some(direction);
            self.direction.send_replace(Some(direction));
        }

        Ok(())
    }
}
