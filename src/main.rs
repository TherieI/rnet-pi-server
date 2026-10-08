use crate::chair::Chair;

mod chair;
mod error;

#[tokio::main]
async fn main() {
    let rsock = openrnet::socket::RnetSock::on_can0(0).unwrap();
    let mut chair = Chair::new(rsock);
    let _ = chair.forward(5000).await;
}
