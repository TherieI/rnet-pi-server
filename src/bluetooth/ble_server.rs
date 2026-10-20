use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicRead, CharacteristicWrite,
        CharacteristicWriteMethod, Descriptor, DescriptorRead, ReqError, Service,
    },
};
use std::collections::BTreeSet;
use std::str;
use tokio::sync::{broadcast::Sender, watch};
use uuid::Uuid;

use crate::messages::messages::{BluetoothMessage, ChairStatus, Mode};

const SERVICE_UUID: Uuid = Uuid::from_u128(0x770412c1297f41e2bb16caeea837ce6c);

const COMMAND_UUID: Uuid = Uuid::from_u128(0xef70b83d67f44d26a9e6c08637548a38);
const STATUS_UUID: Uuid = Uuid::from_u128(0x7e4d647463234be6a4abcf590a1413dd);
const USER_DESCRIPTION_UUID: Uuid = Uuid::from_u128(0x0000290100001000800000805f9b34fb);

pub async fn setup_ble_server(
    publisher: Sender<BluetoothMessage>,
    status: watch::Receiver<ChairStatus>,
) -> Result<(), Box<dyn std::error::Error>> {
    let session = bluer::Session::new().await?;
    let adapter = initialize_adapter(&session).await?;

    run_server(adapter, publisher, status).await
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
    status: watch::Receiver<ChairStatus>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _gatt_handle = adapter
        .serve_gatt_application(create_gatt_application(publisher, status))
        .await?;
    let _advertisement_handle = adapter.advertise(create_advertisement()).await?;

    println!("GATT server started");
    println!("Advertising as SmartWheelchair");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn create_gatt_application(
    publisher: Sender<BluetoothMessage>,
    status: watch::Receiver<ChairStatus>,
) -> Application {
    Application {
        services: vec![Service {
            uuid: SERVICE_UUID,
            primary: true,

            characteristics: vec![
                Characteristic {
                    uuid: COMMAND_UUID,
                    descriptors: vec![user_description(
                        "Chair command: write GUIDE, DOCK, STOP, or DUMB to change mode.",
                    )],

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
                },
                Characteristic {
                    uuid: STATUS_UUID,
                    descriptors: vec![user_description(
                        "Chair status: read JSON containing the current mode and active behavior.",
                    )],
                    read: Some(CharacteristicRead {
                        read: true,
                        fun: Box::new(move |request| {
                            let status = status.clone();
                            Box::pin(async move {
                                status_read_chunk(*status.borrow(), request.offset, request.mtu)
                            })
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],

            ..Default::default()
        }],

        ..Default::default()
    }
}

fn user_description(description: &'static str) -> Descriptor {
    Descriptor {
        uuid: USER_DESCRIPTION_UUID,
        read: Some(DescriptorRead {
            read: true,
            fun: Box::new(move |request| {
                let description = description.as_bytes().to_vec();
                Box::pin(async move { description_read_chunk(&description, request.offset) })
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn description_read_chunk(description: &[u8], offset: u16) -> Result<Vec<u8>, ReqError> {
    let start = usize::from(offset);
    if start > description.len() {
        return Err(ReqError::InvalidOffset);
    }

    Ok(description[start..].to_vec())
}

fn status_read_chunk(status: ChairStatus, offset: u16, mtu: u16) -> Result<Vec<u8>, ReqError> {
    let value = status.to_json().into_bytes();
    let start = usize::from(offset);
    if start > value.len() {
        return Err(ReqError::InvalidOffset);
    }

    let end = (start + usize::from(mtu.saturating_sub(1))).min(value.len());
    Ok(value[start..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characteristic_descriptions_can_be_read_in_chunks() {
        let description = b"Chair command";

        assert_eq!(description_read_chunk(description, 6).unwrap(), b"command");
        assert_eq!(
            description_read_chunk(description, 14),
            Err(ReqError::InvalidOffset)
        );
    }

    #[test]
    fn status_reads_support_ble_offsets() {
        let status = ChairStatus {
            current_mode: Mode::DummyGo,
            current_behavior: "DummyGo",
        };
        let full_status = status.to_json().into_bytes();

        let first_chunk = status_read_chunk(status, 0, 23).unwrap();
        assert_eq!(first_chunk.len(), 22);

        let second_chunk = status_read_chunk(status, 22, 23).unwrap();
        assert_eq!([first_chunk, second_chunk].concat(), full_status);
        assert_eq!(
            status_read_chunk(status, (full_status.len() + 1) as u16, 23),
            Err(ReqError::InvalidOffset)
        );
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
        "DUMB" => Mode::DummyGo,
        _ => {
            println!("Unknown command received: {}", command);
            return;
        }
    };

    let _ = publisher.send(BluetoothMessage::ModeChange(mode));
}
