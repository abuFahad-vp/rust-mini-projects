use proto::calculator_server::{Calculator, CalculatorServer};
use proto::admin_server::{Admin, AdminServer};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("calculator");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = 
        tonic::include_file_descriptor_set!("calculator_descriptor");
}

type State = std::sync::Arc<tokio::sync::RwLock<u64>>;

#[derive(Debug, Default)]
struct AdminService {
    state: State
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _request: tonic::Request<proto::GetCountRequest>
    ) -> Result<tonic::Response<proto::CountResponse>, tonic::Status>{
        let count = self.state.read().await;
        Ok(tonic::Response::new(proto::CountResponse{count: *count}))
    }
}

#[derive(Debug, Default)]
struct CalculatorService {
    state: State
}

impl CalculatorService {
    async fn increment_counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
        println!("Counter: {}",count);
    }
}

#[tonic::async_trait]
impl Calculator for CalculatorService {
    async fn add(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        self.increment_counter().await;
        println!("Got a request: {:?}", request);

        let input = request.get_ref();
        let response = proto::CalculationResponse {
            result: input.a + input.b,
        };

        Ok(tonic::Response::new(response))
    }

    async fn divide(
        &self,
        request: tonic::Request<proto::CalculationRequest>,
    ) -> Result<tonic::Response<proto::CalculationResponse>, tonic::Status> {
        self.increment_counter().await;
        let input = request.get_ref();
        if input.b == 0 {
            return Err(tonic::Status::invalid_argument("Cannot divide by zero"));
        }
        let response = proto::CalculationResponse {
            result: input.a / input.b
        };
        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "[::1]:50051".parse()?;
    let state = State::default();
    let calc = CalculatorService {
        state: state.clone()
    };

    let admin = AdminService {
        state: state.clone()
    };

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    Server::builder()
        .add_service(service)
        .add_service(CalculatorServer::new(calc))
        .add_service(AdminServer::new(admin))
        .serve(addr)
        .await?;
    Ok(())
}