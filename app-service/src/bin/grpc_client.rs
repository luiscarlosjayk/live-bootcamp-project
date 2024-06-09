// pub mod proto {
//     tonic::include_proto!("authentication");
// }
// use dotenvy::dotenv;
// use proto::auth_client::AuthClient;
// use proto::VerifyTokenRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // dotenv().ok();

    // let auth_ip = "[::0]".to_string();
    // let mut client = AuthClient::connect(format!("http://{}:50051", auth_ip)).await?;

    // let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0QHRlc3QuY29tIiwiZXhwIjoxNzE3ODI5ODI5fQ.KD7y0DBVZP5oacOsn2BS-8aFxe_Ys2phCcQu3-3FQ24";
    // let request = tonic::Request::new(VerifyTokenRequest {
    //     token: token.into(),
    // });
    // let response = client.verify_token(request).await?.into_inner();

    // dbg!(response);

    Ok(())
}
