use crate::chair::Chair;

mod chair;
mod error;

#[tokio::main]
async fn main() {
    let rsock = openrnet::socket::RnetSock::on_can0(0).unwrap();
    let mut chair = Chair::new(rsock);
    let _ = chair.set_speed(20);
    let _ = chair.forward(1000).await;
    let _ = chair.set_speed(50);
    let _ = chair.forward(1000).await;
    let _ = chair.set_speed(100);
    let _ = chair.forward(1000).await;
}
