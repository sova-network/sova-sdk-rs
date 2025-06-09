use crate::auth::SovaAuth;
use crate::pem::{TESTNET_CA_PEM, MAINNET_CA_PEM};
use crate::proto::auth::Token;
use crate::searcher::SovaSearcher;

pub struct SovaClient {
    url: String,
    ca_pem: String,
    domain_name: String,
    auth_token: Option<Token>,
}

impl SovaClient {
    pub fn mainnet() -> Self {
        Self::mainnet_with_auth(None)
    }

    pub fn mainnet_with_auth(auth_token: Option<Token>) -> Self {
        Self::custom(
            "https://engine.sova.network:30020",
            MAINNET_CA_PEM,
            "engine.sova.network",
            auth_token,
        )
    }

    pub fn testnet() -> Self {
        Self::testnet_with_auth(None)
    }

    pub fn testnet_with_auth(auth_token: Option<Token>) -> Self {
        Self::custom(
            "https://testnet-engine.sova.network:30020",
            TESTNET_CA_PEM,
            "testnet-engine.sova.network",
            auth_token,
        )
    }

    pub fn custom(url: &str, ca_pem: &str, domain_name: &str, auth_token: Option<Token>) -> Self {
        Self {
            url: url.to_owned(),
            ca_pem: ca_pem.to_owned(),
            domain_name: domain_name.to_owned(),
            auth_token,
        }
    }

    pub async fn authenticate(
        &mut self,
        private_key: [u8; 32],
    ) -> Result<Token, Box<dyn std::error::Error>> {
        let mut auth = SovaAuth::new(
            Box::leak(self.url.clone().into_boxed_str()),
            Some(&self.ca_pem),
            Some(&self.domain_name),
            &private_key,
        )
        .await?;

        auth.authenticate().await?;

        let token = auth
            .access_token()
            .ok_or("Authentication failed: missing access token")?;

        self.auth_token = Some(token.clone());

        Ok(token)
    }

    pub async fn searcher(&self) -> Result<SovaSearcher, Box<dyn std::error::Error>> {
        SovaSearcher::new_with_access_token(
            Box::leak(self.url.clone().into_boxed_str()),
            Some(&self.ca_pem),
            Some(&self.domain_name),
            self.auth_token.clone(),
        )
        .await
    }
}
