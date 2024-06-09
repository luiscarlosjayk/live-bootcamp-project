// use crate::app_state::BannedTokenStoreType;
// use crate::utils::auth;
// use proto::auth_server::Auth;
// use proto::{StatusCode, VerifyTokenRequest, VerifyTokenResponse};
// use tonic::{Request, Response, Status};

// pub mod proto {
//     tonic::include_proto!("authentication");
// }

// // Re-exporting
// pub use proto::auth_server::AuthServer;

// pub struct AuthService {
//     pub banned_token_store: BannedTokenStoreType,
// }

// impl AuthService {
//     pub fn new(banned_token_store: BannedTokenStoreType) -> Self {
//         Self { banned_token_store }
//     }
// }

// #[tonic::async_trait]
// impl Auth for AuthService {
//     async fn verify_token(
//         &self,
//         request: Request<VerifyTokenRequest>,
//     ) -> Result<Response<VerifyTokenResponse>, Status> {
//         println!("Got a grpc request: {:?}", request);

//         let token = request.into_inner().token;
//         match auth::validate_token(&token, self.banned_token_store.clone()).await {
//             Ok(_) => Ok(Response::new(VerifyTokenResponse {
//                 status: StatusCode::Ok.into(),
//             })),
//             Err(_) => Err(Status::unauthenticated("The token is not valid")),
//         }
//     }
// }
