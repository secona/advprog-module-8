use services::chat_service_server::{ChatService, ChatServiceServer};
use services::payment_service_server::{PaymentService, PaymentServiceServer};
use services::transaction_service_server::{TransactionService, TransactionServiceServer};
use services::{ChatMessage, PaymentRequest, PaymentResponse, TransactionRequest, TransactionResponse};
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic::transport::Server;

pub mod services {
    tonic::include_proto!("services");
}

#[derive(Default)]
pub struct MyPaymentService;

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(
        &self,
        request: Request<PaymentRequest>
    ) -> Result<Response<PaymentResponse>, Status> {
        println!("Received payment request: {:?}", request);
        Ok(Response::new(PaymentResponse { success: true }))
    }
}

#[derive(Default)]
pub struct MyTransactionService;

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(
        &self,
        request: Request<TransactionRequest>
    ) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        println!("Received transaction history request: {:?}", request);
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..30 {
                if tx.send(Ok(TransactionResponse {
                    transaction_id: format!("trans_{}", i),
                    status: "Completed".to_string(),
                    amount: 100.0,
                    timestamp: "2022-01-01T12:00:00Z".to_string(),
                })).await.is_err() {
                    break;
                }

                if i % 10 == 9 {
                    time::sleep(time::Duration::from_secs(1)).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Default)]
pub struct MyChatService;

#[tonic::async_trait]
impl ChatService for MyChatService {
    type ChatStream = ReceiverStream<Result<ChatMessage, Status>>;

    async fn chat(
        &self, 
        request: Request<Streaming<ChatMessage>>
    ) -> Result<Response<Self::ChatStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(message) = stream.message().await.unwrap_or(None) {
                println!("Received Message: {:?}", message);

                let reply = ChatMessage {
                    user_id: message.user_id.clone(),
                    message: format!("Pesan Anda akan dibalas pada jam kerja. Pesan Anda: {}", message.message),
                };

                let _ = tx.send(Ok(reply)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let payment_service = MyPaymentService;
    let transaction_service = MyTransactionService;
    let chat_service = MyChatService;

    Server::builder()
        .add_service(PaymentServiceServer::new(payment_service))
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(ChatServiceServer::new(chat_service))
        .serve(addr)
        .await?;

    Ok(())
}
