use openrnet::RnetSock;

use crate::chair::Chair;

mod chair;
mod error;

#[tokio::main]
async fn main() {
    let mut chair = Chair::new(RnetSock::on_can0(0).unwrap());
    let _ = chair.forward(100).await;
}
