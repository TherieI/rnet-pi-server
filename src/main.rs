extern crate openrnet;

use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application,
        Characteristic,
        CharacteristicWrite,
        CharacteristicWriteMethod,
        Service,
    },
};
use std::time::Duration;
use tokio::time::sleep;


fn main() {
    //let _rsock = openrnet::socket::RnetSock::new("vcan0").unwrap();
    test();
}



#[tokio::main]
async fn test() -> Result<(), Box<dyn std::error::Error>>{
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    println!("bluetooth adapter {} is ready.", adapter.name());

    let le_advertisement = Advertisement {
        local_name: Some("TestPI,Reciever".to_string()),
        discoverable: Some(true),
        service_uuids: vec![].into_iter().collect(),
        ..Default::default()
    };
    let _adv_handle = adapter.advertise(le_advertisement).await?;
    println!("advertising started...");

    let custom_characteristic = Characteristic {
        uuid: "87654321-4321-8765-4321-876543210987".parse()?,
        write: Some(CharacteristicWrite {
            write: true,
            write_without_response: true,
            method: CharacteristicWriteMethod::Fun(Box::new(move |value, _| {
                Box::pin(async move{
                    if let Ok(recieved_str) = String::from_utf8(value.clone()) {
                        println!("Recieved String: {}", recieved_str);
                    } else{
                        println!("Received Raw Bytes: {:?}", value);
                    }
                    Ok(())
                })
            })),
            ..Default::default()
        }),
        ..Default::default()
    };

    let custom_service = Service {
        uuid: "12345678-1234-5678-1234-567812345678".parse()?,
        primary: true,
        characteristics: vec![custom_characteristic],
        ..Default::default()
    };

    let mut app = Application::default();
    app.services.push(custom_service);

    let _app_handle = adapter.serve_gatt_application(app).await?;
    println!("Gatt Server is active and listening for data.");

    loop{
        sleep(Duration::from_secs(3600)).await;
    }

}