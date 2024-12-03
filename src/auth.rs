use k256::ecdsa::{signature::Signer, Signature};
use k256::ecdsa::{SigningKey, VerifyingKey};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

use crate::error::MevtonError;
use crate::proto::auth::auth_service_client::AuthServiceClient;
use crate::proto::auth::Token;
use crate::proto::auth::{
    GenerateAuthChallengeRequest, GenerateAuthTokensRequest, RefreshAccessTokenRequest,
};

pub struct NewKeyPair {
    private_key: SigningKey,
    public_key: VerifyingKey,
}

pub struct MevtonAuth {
    auth_client: AuthServiceClient<Channel>,
    key: NewKeyPair,
    access_token: Option<Token>,
    refresh_token: Option<Token>,
}

impl MevtonAuth {
    pub async fn new(
        url: &'static str,
        ca_pem: Option<&str>,
        domain_name: Option<&str>,
        private_key: &[u8; 32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_client = if let (Some(ca_pem), Some(domain_name)) = (ca_pem, domain_name) {
            let ca = Certificate::from_pem(ca_pem);

            let tls = ClientTlsConfig::new()
                .ca_certificate(ca)
                .domain_name(domain_name);

            let channel = Channel::from_static(url).tls_config(tls)?.connect().await?;

            AuthServiceClient::new(channel)
        } else {
            AuthServiceClient::connect(url).await?
        };

        let private_key = SigningKey::from_slice(private_key).map_err(Box::new)?;
        let public_key = VerifyingKey::from(&private_key);
        let key = NewKeyPair {
            private_key,
            public_key,
        };

        Ok(Self {
            auth_client,
            key,
            access_token: None,
            refresh_token: None,
        })
    }

    pub async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes_public_key: &[u8] = &self.key.public_key.to_sec1_bytes();

        let request = tonic::Request::new(GenerateAuthChallengeRequest {
            pubkey: Vec::from(bytes_public_key),
        });
        let response = self.auth_client.generate_auth_challenge(request).await?;

        let challenge = response.into_inner().challenge;
        let signed_challenge: Signature = self.key.private_key.sign(&challenge);

        let token_request = tonic::Request::new(GenerateAuthTokensRequest {
            challenge,
            signed_challenge: signed_challenge.to_vec(),
        });

        let token_response = self
            .auth_client
            .generate_auth_tokens(token_request)
            .await?
            .into_inner();
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

            return Ok(());
        }

        Err(Box::from(MevtonError::AuthenticationRequired))
    }

    pub fn access_token(&self) -> Option<Token> {
        self.access_token.clone()
    }

    pub fn refresh_token(&self) -> Option<Token> {
        self.refresh_token.clone()
    }
}
