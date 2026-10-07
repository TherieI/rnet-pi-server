extern crate openrnet;

fn main() {
    let rsock = openrnet::socket::RnetSock::new("vcan0").unwrap();
    bluetooth();
}

use bluer::{
    adv::Advertisement,
    gatt::local::{
        characteristic_control,
        Application,
        Characteristic,
        CharacteristicControlEvent,
        CharacteristicWrite,
        CharacteristicWriteMethod,
        Service,
    },
};
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};

use std::collections::BTreeSet;

// Our custom BLE service UUID.
const SERVICE_UUID: uuid::Uuid =
    uuid::uuid!("12345678-1234-5678-1234-56789abcdef0");

// The characteristic that receives messages.
const MESSAGE_UUID: uuid::Uuid =
    uuid::uuid!("12345678-1234-5678-1234-56789abcdef1");


async fn bluetooth() -> bluer::Result<()> {
    env_logger::init();

    // Connect to Linux's Bluetooth system.
    let session = bluer::Session::new().await?;

    // Get the default Bluetooth adapter.
    let adapter = session.default_adapter().await?;

    adapter.set_powered(true).await?;

    println!("Bluetooth adapter: {}", adapter.name());
    println!("Bluetooth address: {}", adapter.address().await?);

    // ------------------------------------------------------------
    // Advertise the Pi
    // ------------------------------------------------------------

    let advertisement = Advertisement {
        service_uuids: BTreeSet::from([SERVICE_UUID]),
        discoverable: Some(true),
        local_name: Some("Pi-BLE".to_string()),
        ..Default::default()
    };

    let advertisement_handle = adapter.advertise(advertisement).await?;

    println!("Advertising as Pi-BLE");

    // ------------------------------------------------------------
    // Create our GATT characteristic
    // ------------------------------------------------------------

    let (characteristic_control, characteristic_handle) =
        characteristic_control();

    let app = Application {
        services: vec![
            Service {
                uuid: SERVICE_UUID,
                primary: true,

                characteristics: vec![
                    Characteristic {
                        uuid: MESSAGE_UUID,

                        // Allow the iPhone to write messages.
                        write: Some(CharacteristicWrite {
                            write: true,
                            write_without_response: true,

                            // We'll receive writes through the control
                            // event stream below.
                            method: CharacteristicWriteMethod::Io,

                            ..Default::default()
                        }),

                        control_handle: characteristic_handle,

                        ..Default::default()
                    }
                ],

                ..Default::default()
            }
        ],

        ..Default::default()
    };

    // Register the GATT application with BlueZ.
    let _app_handle =
        adapter.serve_gatt_application(app).await?;

    println!("GATT server started.");
    println!("Waiting for iPhone...");
    println!();
    println!("Service UUID:");
    println!("{}", SERVICE_UUID);
    println!();
    println!("Message characteristic:");
    println!("{}", MESSAGE_UUID);
    println!();

    // ------------------------------------------------------------
    // Wait for messages
    // ------------------------------------------------------------

    tokio::pin!(characteristic_control);

    loop {
        match characteristic_control.next().await {
            Some(CharacteristicControlEvent::Write(request)) => {
                println!("iPhone connected!");

                // Accept the write request.
                let mut reader = request.accept()?;

                let mut buffer = vec![0u8; request.mtu()];

                // Read the incoming data.
                loop {
                    match tokio::io::AsyncReadExt::read(
                        &mut reader,
                        &mut buffer,
                    )
                    .await
                    {
                        Ok(0) => {
                            println!("iPhone disconnected.");
                            break;
                        }

                        Ok(length) => {
                            let data = &buffer[..length];

                            match std::str::from_utf8(data) {
                                Ok(message) => {
                                    println!(
                                        "Received: {}",
                                        message
                                    );
                                }

                                Err(_) => {
                                    println!(
                                        "Received binary data: {:02X?}",
                                        data
                                    );
                                }
                            }
                        }

                        Err(error) => {
                            eprintln!(
                                "Bluetooth read error: {}",
                                error
                            );
                            break;
                        }
                    }
                }
            }

            Some(_) => {
                // Ignore other events.
            }

            None => {
                println!("Bluetooth characteristic closed.");
                break;
            }
        }
    }

    drop(advertisement_handle);

    Ok(())
}
