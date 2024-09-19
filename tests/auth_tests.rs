use k256::ecdsa::SigningKey;
use tonic::{Response, Status};

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tonic::transport::Server;
use mevton_rs::auth::MevtonAuth;
use mevton_rs::proto::auth::{GenerateAuthChallengeRequest, GenerateAuthChallengeResponse, GenerateAuthTokensRequest, GenerateAuthTokensResponse, RefreshAccessTokenRequest, RefreshAccessTokenResponse, Token};
use mevton_rs::proto::auth::auth_service_server::{AuthService, AuthServiceServer};


#[tokio::test]
async fn test_authenticate() -> Result<(), Box<dyn std::error::Error>> {
    // Set up a mock auth service
    struct MockAuthService;

    #[tonic::async_trait]
    impl AuthService for MockAuthService {
        async fn generate_auth_challenge(
            &self,
            _request: tonic::Request<GenerateAuthChallengeRequest>,
        ) -> Result<tonic::Response<GenerateAuthChallengeResponse>, tonic::Status> {
            Ok(Response::new(GenerateAuthChallengeResponse {
                challenge: b"test_challenge".to_vec(),
            }))
        }

        async fn generate_auth_tokens(
            &self,
            _request: tonic::Request<GenerateAuthTokensRequest>,
        ) -> Result<tonic::Response<GenerateAuthTokensResponse>, tonic::Status> {
            Ok(Response::new(GenerateAuthTokensResponse {
                access_token: Some(Token {
                    value: "access_token".to_string(),
                    expires_at_utc: None,
                }),
                refresh_token: Some(Token {
                    value: "refresh_token".to_string(),
                    expires_at_utc: None,
                }),
            }))
        }

        async fn refresh_access_token(
            &self,
            _request: tonic::Request<RefreshAccessTokenRequest>,
        ) -> Result<tonic::Response<RefreshAccessTokenResponse>, tonic::Status> {
            Ok(Response::new(RefreshAccessTokenResponse {
                access_token: Some(Token {
                    value: "new_access_token".to_string(),
                    expires_at_utc: None,
                }),
            }))
        }
    }

    // Start mock server
    let addr = "[::1]:50051".parse().unwrap();
    let svc = AuthServiceServer::new(MockAuthService);

    let (tx, mut rx) = mpsc::channel(1);
    let server_handle: JoinHandle<()> = tokio::spawn(async move {
        // Notify that the server is starting
        tx.send(()).await.unwrap();
        // Start the server (this will block until the server is shut down)
        Server::builder().add_service(svc).serve(addr).await.unwrap();
    });

    // Wait for the server to start
    rx.recv().await;



    // Private key for testing
    let private_key_bytes: [u8; 32] = [155, 202, 118, 43, 82, 100, 113, 150, 99, 21, 45, 230, 88, 247, 193, 12, 92, 78, 191, 229, 73, 191, 100, 156, 231, 41, 144, 54, 202, 199, 75, 1];
    // A dummy key, replace with real one for actual tests

    // Create MevtonAuth instance
    let mut auth = MevtonAuth::new("http://[::1]:50051",&private_key_bytes).await?;

    // Test authenticate function
    auth.authenticate().await?;

    // Assert that the tokens were set correctly
    assert_eq!(auth.access_token().unwrap().value, "access_token");
    assert_eq!(auth.refresh_token().unwrap().value, "refresh_token");

    // Stop the server
    server_handle.abort();

    Ok(())
}
