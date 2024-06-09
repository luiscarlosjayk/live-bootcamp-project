use auth_service::services::{grpc_auth, hashset_banned_token_store::HashsetBannedTokenStore};
use dotenvy::dotenv;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), tonic::transport::Error> {
    dotenv().ok();

    let addr = "[::0]:50051"
        .parse()
        .map_err(|_| tonic::Status::internal("Could not parse grpc address"));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let grpc_auth_service = grpc_auth::AuthService::new(banned_token_store);

    println!("Running grpc server on: {:?}", addr);
    tonic::transport::Server::builder()
        .add_service(grpc_auth::AuthServer::new(grpc_auth_service))
        .serve(addr.unwrap())
        .await
}
