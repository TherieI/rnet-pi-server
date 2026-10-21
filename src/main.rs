use openrnet::RnetSock;

use crate::chair::Chair;

mod chair;
mod error;

#[tokio::main]
async fn main() {
    let mut chair = Chair::new(RnetSock::on_can0(0).unwrap());
    let _ = chair.forward(2000).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let _ = chair.honk(100).await;
    let _ = chair.forward(1000).await;
}
