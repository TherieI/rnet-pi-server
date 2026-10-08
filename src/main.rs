use std::time::Duration;

use crate::{chair::Chair, error::ChairError};

mod chair;
mod error;

#[tokio::main]
async fn main() -> Result<(), ChairError> {
    let rsock = openrnet::socket::RnetSock::on_can0(0).unwrap();
    let mut chair = Chair::new(rsock);
    // chair.honk(1000).await?;
    println!("setting speed to 20");
    chair.set_speed(20).await?;
    println!("moving forward");
    chair.forward(1000).await?;
    // chair.honk(500).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("setting speed to 50");
    chair.set_speed(50).await?;
    println!("moving forward");
    chair.forward(1000).await?;
    // chair.honk(500).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("setting speed to MAX");
    chair.set_speed(100).await?;
    println!("moving forward");
    chair.forward(1000).await?;
    Ok(())
}
