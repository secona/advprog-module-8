use services::chat_service_client::ChatServiceClient;
use services::{ChatMessage, TransactionRequest};
use services::{transaction_service_client::TransactionServiceClient, PaymentRequest};
use services::payment_service_client::PaymentServiceClient;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
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

    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let mut client = ChatServiceClient::new(channel);

    let (tx, rx) = mpsc::channel::<ChatMessage>(32);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            let message = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };

            if (tx.send(message).await).is_err() {
                eprintln!("Failed to send message!");
                break;
            }
        }
    });

    let request = Request::new(ReceiverStream::new(rx));
    let mut stream = client.chat(request).await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Server says: {:?}", response);
    }

    Ok(())
}
