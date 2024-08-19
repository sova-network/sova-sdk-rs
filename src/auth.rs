use tonic::transport::Channel;

use fastcrypto::traits::{KeyPair, ToFromBytes};
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::traits::Signer;

use crate::error::MevtonError;
use crate::proto::auth::Token;
use crate::proto::auth::auth_service_client::AuthServiceClient;
use crate::proto::auth::{GenerateAuthChallengeRequest, GenerateAuthTokensRequest, RefreshAccessTokenRequest};

pub struct MevtonAuth {
    auth_client: AuthServiceClient<Channel>,
    key: Ed25519KeyPair,
    access_token: Option<Token>,
    refresh_token: Option<Token>
}

impl MevtonAuth {
    pub async fn new(auth_url: &str, private_key: &[u8; 32]) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_client = AuthServiceClient::connect(auth_url.to_string()).await?;
        let key = Ed25519KeyPair::from_bytes(private_key)?;

        Ok(Self {
            auth_client,
            key,
            access_token: None,
            refresh_token: None
        })
    }

    pub async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GenerateAuthChallengeRequest {
            pubkey: Vec::from(self.key.public().as_bytes()),
        });

        let response = self.auth_client.generate_auth_challenge(request).await?;
        let challenge = response.into_inner().challenge;

        let signed_challenge = self.key.sign(&challenge);

        let token_request = tonic::Request::new(GenerateAuthTokensRequest {
            challenge,
            signed_challenge: Vec::from(signed_challenge.sig.to_bytes()),
        });

        let token_response = self.auth_client.generate_auth_tokens(token_request).await?.into_inner();
        self.access_token = token_response.access_token;
        self.refresh_token = token_response.refresh_token;

        Ok(())
    }

    pub async fn refresh_access_token(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(refresh_token) = &self.refresh_token {
            let request = tonic::Request::new(RefreshAccessTokenRequest {
                refresh_token: refresh_token.value.clone(),
            });

            let response = self.auth_client.refresh_access_token(request).await?;
            self.access_token = response.into_inner().access_token;

            return Ok(())
        }

        return Err(Box::from(MevtonError::AuthenticationRequired));
    }

    pub fn access_token(&self) -> Option<Token> {
        self.access_token.clone()
    }

    pub fn refresh_token(&self) -> Option<Token> {
        self.refresh_token.clone()
    }
}
