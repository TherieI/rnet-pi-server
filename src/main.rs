mod bluetooth;
mod messages;
mod orchestrator;

use tokio::sync::broadcast;

use messages::messages::BluetoothMessage;

#[tokio::main]
async fn main() {
    //let rsock = openrnet::socket::RnetSock::new("vcan0").unwrap();
    let (tx, _) = broadcast::channel::<BluetoothMessage>(32);
    let orchestrator_rx = tx.subscribe();

    let ble_tx = tx.clone();

    tokio::spawn(async move {
        if let Err(error) = bluetooth::ble_server::setup_ble_server(ble_tx).await {
            eprintln!("BLE server failed: {error}");
        }
    });

    orchestrator::behavior_orchestrator::BehaviorOrchestrator::new()
        .run(orchestrator_rx)
        .await;
}