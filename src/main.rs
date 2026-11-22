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

    let chair_task = tokio::spawn(async move { controller.run(joystick_rx).await });

    let ble_tx = tx.clone();

    let ble_task =
        tokio::spawn(
            async move { bluetooth::ble_server::setup_ble_server(ble_tx, status_rx).await },
        );

    let orchestrator_task = orchestrator::behavior_orchestrator::BehaviorOrchestrator::new(
        status_tx,
        joystick_tx.clone(),
    )
    .run(orchestrator_rx);

    tokio::pin!(chair_task);
    tokio::pin!(ble_task);
    tokio::pin!(orchestrator_task);

    tokio::select! {
        result = &mut chair_task => {
            joystick_tx.send(JoystickCommand { x: 0, y: 0 }).ok();
            match result {
                Ok(Ok(())) => eprintln!("Chair controller stopped"),
                Ok(Err(error)) => eprintln!("Chair controller failed: {error}"),
                Err(error) => eprintln!("Chair controller task panicked: {error}"),
            }
        }
        result = &mut ble_task => {
            joystick_tx.send(JoystickCommand { x: 0, y: 0 }).ok();
            match result {
                Ok(Ok(())) => eprintln!("BLE server stopped"),
                Ok(Err(error)) => eprintln!("BLE server failed: {error}"),
                Err(error) => eprintln!("BLE server task panicked: {error}"),
            }
        }
        () = &mut orchestrator_task => {
            joystick_tx.send(JoystickCommand { x: 0, y: 0 }).ok();
            eprintln!("Behavior orchestrator stopped");
        }
    }

    chair_task.abort();
    ble_task.abort();
    Ok(())
}
