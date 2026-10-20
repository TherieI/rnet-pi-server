mod behaviors;
mod bluetooth;
mod messages;
mod orchestrator;

use tokio::sync::broadcast;

use messages::messages::{BluetoothMessage, ChairStatus, Mode};

#[tokio::main]
async fn main() {
    //let rsock = openrnet::socket::RnetSock::new("vcan0").unwrap();
    let (tx, _) = broadcast::channel::<BluetoothMessage>(32);
    let orchestrator_rx = tx.subscribe();
    let (status_tx, status_rx) = tokio::sync::watch::channel(ChairStatus {
        current_mode: Mode::EmergencyStop,
        current_behavior: "EmergencyStop",
    });

    let ble_tx = tx.clone();

    tokio::spawn(async move {
        if let Err(error) = bluetooth::ble_server::setup_ble_server(ble_tx, status_rx).await {
            eprintln!("BLE server failed: {error}");
        }
    });

    orchestrator::behavior_orchestrator::BehaviorOrchestrator::new(status_tx)
        .run(orchestrator_rx)
        .await;
}
