use std::error::Error;
use proto::calculator_client::CalculatorClient;
use proto::admin_client::AdminClient;

pub mod proto {
    tonic::include_proto!("calculator");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";
    let mut client = CalculatorClient::connect(url).await?;
    let mut client2 = AdminClient::connect(url).await?;

    let req = proto::CalculationRequest {a: 4, b: 1};
    let request = tonic::Request::new(req);
    let response = client.divide(request).await?;

    let count_req = proto::GetCountRequest{};
    let count_request = tonic::Request::new(count_req);
    let count_res = client2.get_request_count(count_request).await?;

    println!("Response: {:?}", response.get_ref().result);
    println!("Response: {:?}", count_res.get_ref().count);
    Ok(())
}