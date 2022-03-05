# sensor-grpc-adapter

Rust Adapter to interact with sensor implementation via GRPC. 
Sensor data is send over GRPC Endpoint. 

## Build

```
cargo build or cross build --target <target>
cargo doc --open
```

Tested:
* cargo v1.56.0
* rustc v1.56.1
* stable
* cross 0.2.1

Architecture:
* x86_64-unknown-linux-gnu
* armv7-unknown-linux-gnueabihf
* aarch64-unknown-linux-gnu

## Install

```
[dependencies]
sensor-grpc-adapter = { git = "https://github.com/fprotopapa/sensor-grpc-adapter.git", branch = "main" }
```

## Example

server.rs

```
use sensor_grpc_adapter as adapter;

use tokio::join;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (service, rx) = adapter::SensorAdapterService::new(32);
    let sensor_worker = sensor_state_machine(rx);
    let grpc_server = adapter::run_sensor_adapter_server(service, "[::1]:50051");
    let _result = join!(sensor_worker, grpc_server);
    Ok(())
}

async fn sensor_state_machine(mut rx: mpsc::Receiver<sensor_grpc_adapter::ServerSensorChannel>) {
    loop {
        let request = match rx.recv().await {
            Some(msg) => {
                println!("Data: {:?}", msg.data);
                msg
            }
            None => panic!("Received No Data"),
        };
        let _res = request.tx.send(adapter::SensorReply {
            status: "Received Data".to_string(),
            command: "".to_string(),
            payload: "".to_string(),
        });
    }
}
```

client.rs

```
use sensor_grpc_adapter as adapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = adapter::connect_sensor_adapter_client("[::1]:50051").await?;
    let response = adapter::send_sensor_data(
        &mut client,
        adapter::SensorData {
            sensor_id: "123".to_string(),
            name: "test".to_string(),
            sensor_type: "temperature".to_string(),
            value: "12".to_string(),
            unit: "C".to_string(),
            timestamp: 1233425346 as i64,
            command: "".to_string(),
        },
    )
    .await?;
    println!("Response: {:?}", response);
    Ok(())
}
```