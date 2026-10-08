use crate::{chair::Chair, error::ChairError};

mod chair;
mod error;

#[tokio::main]
async fn main() -> Result<(), ChairError> {
    let rsock = openrnet::socket::RnetSock::on_can0(0).unwrap();
    let mut chair = Chair::new(rsock);
    chair.set_speed(20).await?;
    chair.forward(1000).await?;
    // chair.honk(500).await?;
    chair.set_speed(50).await?;
    chair.forward(1000).await?;
    // chair.honk(500).await?;
    chair.set_speed(100).await?;
    chair.forward(1000).await?;
    Ok(())
}
