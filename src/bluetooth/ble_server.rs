use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicWrite, CharacteristicWriteMethod, Service,
    },
};
use std::collections::BTreeSet;
use std::str;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

use crate::messages::messages::{BluetoothMessage, Mode};

const SERVICE_UUID: Uuid = Uuid::from_u128(0x770412c1297f41e2bb16caeea837ce6c);

const COMMAND_UUID: Uuid = Uuid::from_u128(0xef70b83d67f44d26a9e6c08637548a38);

pub async fn setup_ble_server(
    publisher: Sender<BluetoothMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let session = bluer::Session::new().await?;
    let adapter = initialize_adapter(&session).await?;

    run_server(adapter, publisher).await
}

async fn initialize_adapter(
    session: &bluer::Session,
) -> Result<bluer::Adapter, Box<dyn std::error::Error>> {
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    println!("Bluetooth adapter '{}' ready", adapter.name());

    Ok(adapter)
}

async fn run_server(
    adapter: bluer::Adapter,
    publisher: Sender<BluetoothMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _gatt_handle = adapter
        .serve_gatt_application(create_gatt_application(publisher))
        .await?;
    let _advertisement_handle = adapter.advertise(create_advertisement()).await?;

    println!("GATT server started");
    println!("Advertising as SmartWheelchair");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn create_gatt_application(publisher: Sender<BluetoothMessage>) -> Application {
    Application {
        services: vec![Service {
            uuid: SERVICE_UUID,
            primary: true,

            characteristics: vec![Characteristic {
                uuid: COMMAND_UUID,

                write: Some(CharacteristicWrite {
                    write: true,
                    write_without_response: true,

                    method: CharacteristicWriteMethod::Fun(Box::new(move |value, _req| {
                        let publisher = publisher.clone();

                        Box::pin(async move {
                            println!("Raw BLE write: {:?}", value);

                            if let Ok(text) = str::from_utf8(&value) {
                                handle_command(text.trim(), &publisher);
                            }

                            Ok(())
                        })
                    })),

                    ..Default::default()
                }),

                ..Default::default()
            }],

            ..Default::default()
        }],

        ..Default::default()
    }
}

fn create_advertisement() -> Advertisement {
    Advertisement {
        discoverable: Some(true),
        local_name: Some("SmartWheelchair".to_string()),
        service_uuids: BTreeSet::from([SERVICE_UUID]),
        ..Default::default()
    }
}

fn handle_command(command: &str, publisher: &Sender<BluetoothMessage>) {
    let mode = match command {
        "GUIDE" => Mode::GuideFollowing,
        "DOCK" => Mode::AutoDock,
        "STOP" => Mode::EmergencyStop,
        _ => {
            println!("Unknown command received: {}", command);
            return;
        }
    };

    let _ = publisher.send(BluetoothMessage::ModeChange(mode));
}
