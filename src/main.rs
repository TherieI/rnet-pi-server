mod behaviors;
mod bluetooth;
mod chair;
mod messages;
mod orchestrator;

use tokio::sync::{broadcast, watch};

use behaviors::behavior::JoystickCommand;
use chair::{chair_controller::ChairController, error::ChairError};
use messages::messages::{BluetoothMessage, ChairStatus, Mode};

#[tokio::main]
async fn main() -> Result<(), ChairError> {
    let controller = if std::env::var_os("RNET_MOCK_CHAIR").is_some() {
        let (controller, _observed_direction) = ChairController::new_mock();
        controller
    } else {
        ChairController::new()?
    };
    let (tx, _) = broadcast::channel::<BluetoothMessage>(32);
    let orchestrator_rx = tx.subscribe();
    let (status_tx, status_rx) = tokio::sync::watch::channel(ChairStatus {
        current_mode: Mode::EmergencyStop,
        current_behavior: "EmergencyStop",
    });
    let (joystick_tx, joystick_rx) = watch::channel(JoystickCommand { x: 0, y: 0 });

    tokio::spawn(async move {
        if let Err(error) = controller.run(joystick_rx).await {
            eprintln!("Chair controller failed: {error}");
        }
    });

    let ble_tx = tx.clone();

    tokio::spawn(async move {
        if let Err(error) = bluetooth::ble_server::setup_ble_server(ble_tx, status_rx).await {
            eprintln!("BLE server failed: {error}");
        }
    });

    orchestrator::behavior_orchestrator::BehaviorOrchestrator::new(status_tx, joystick_tx)
        .run(orchestrator_rx)
        .await;
    Ok(())
}
