use grpc_sensor::sensor_adapter_client::SensorAdapterClient;
use grpc_sensor::sensor_adapter_server::{SensorAdapter, SensorAdapterServer};
use grpc_sensor::{SensorAdapterReply, SensorDataRequest};
use tokio::sync::{mpsc, oneshot};
use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Status};
/// Protobuffer v3 file
pub mod grpc_sensor {
    tonic::include_proto!("sensor_adapter_grpc");
}

#[derive(Debug)]
pub struct ServerSensorChannel {
    pub data: SensorData,
    pub tx: oneshot::Sender<SensorReply>,
}

#[derive(Debug)]
pub struct SensorData {
    pub sensor_id: String,
    pub name: String,
    pub sensor_type: String,
    pub value: String,
    pub unit: String,
    pub timestamp: i64,
    pub command: String,
}

#[derive(Debug)]
pub struct SensorReply {
    pub status: String,
    pub command: String,
    pub payload: String,
}
/// Structure for Implementing GRPC Calls
#[derive(Debug)]
pub struct SensorAdapterService {
    pub tx: mpsc::Sender<ServerSensorChannel>,
}

impl SensorAdapterService {
    pub fn new(buffer_size: usize) -> (SensorAdapterService, mpsc::Receiver<ServerSensorChannel>) {
        let (tx, rx) = mpsc::channel::<ServerSensorChannel>(buffer_size);
        (SensorAdapterService { tx: tx }, rx)
    }
}
/// Implementation of GRPC Calls
/// send_sensor_data, send_command
#[tonic::async_trait]
impl SensorAdapter for SensorAdapterService {
    async fn send_sensor_data(
        &self,
        request: Request<SensorDataRequest>,
    ) -> Result<Response<SensorAdapterReply>, Status> {
        let request = request.into_inner();
        println!("send_sensor_data: {:?}", request);
        let tx = self.tx.clone();
        let (tx_one, rx_one) = oneshot::channel();
        let _res = tx
            .send(ServerSensorChannel {
                data: SensorData {
                    sensor_id: request.sensor_id,
                    name: request.name,
                    sensor_type: request.sensor_type,
                    value: request.value,
                    unit: request.unit,
                    timestamp: request.timestamp,
                    command: request.command,
                },
                tx: tx_one,
            })
            .await;
        let response = match rx_one.await {
            Ok(res) => res,
            Err(_e) => return Err(Status::cancelled("Error Receiving Messsages")),
        };
        Ok(Response::new(SensorAdapterReply {
            status: response.status,
            command: response.command,
            payload: response.payload,
        }))
    }
}

pub async fn run_sensor_adapter_server(
    service: SensorAdapterService,
    socket: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = socket.parse()?;
    // Start thread
    let _grpc_server = Server::builder()
        .add_service(SensorAdapterServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

pub async fn connect_sensor_adapter_client(
    socket: &str,
) -> Result<SensorAdapterClient<Channel>, Box<dyn std::error::Error>> {
    let client = SensorAdapterClient::connect(format!("http://{}", socket)).await?;
    Ok(client)
}

pub async fn send_sensor_data(
    client: &mut SensorAdapterClient<Channel>,
    data: SensorData,
) -> Result<SensorReply, Status> {
    let msg = SensorDataRequest {
        sensor_id: data.sensor_id,
        name: data.name,
        sensor_type: data.sensor_type,
        value: data.value,
        unit: data.unit,
        timestamp: data.timestamp,
        command: data.command,
    };
    let response = match client.send_sensor_data(tonic::Request::new(msg)).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    let response = response.into_inner();
    Ok(SensorReply {
        status: response.status,
        command: response.command,
        payload: response.payload,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
