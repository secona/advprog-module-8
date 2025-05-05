use services::TransactionRequest;
use services::{transaction_service_client::TransactionServiceClient, PaymentRequest};
use services::payment_service_client::PaymentServiceClient;
use tonic::Request;

pub mod services {
    tonic::include_proto!("services");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaymentServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 100.0,
    });

    let response = client.process_payment(request).await?;
    println!("RESPONSE={:?}", response.into_inner());

    let mut client = TransactionServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(TransactionRequest {
        user_id: "user_123".to_string(),
    });

    let mut stream = client.get_transaction_history(request).await?.into_inner();
    while let Some(transaction) = stream.message().await? {
        println!("Transaction: {:?}", transaction);
    }

    Ok(())
}
