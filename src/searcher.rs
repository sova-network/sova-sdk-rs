use std::str::FromStr;
use tonic::transport::Channel;

use crate::proto;

use crate::proto::auth::Token;
use crate::proto::searcher::searcher_service_client::SearcherServiceClient;
use crate::proto::searcher::{AddressSubscriptionV0, GetTipAddressesRequest, GetTipAddressesResponse, MempoolSubscription, SendBundleResponse};

pub struct MevtonSearcher {
    searcher_client: SearcherServiceClient<Channel>,
    access_token: Option<Token>,
}

impl MevtonSearcher {
    pub async fn new(searcher_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let searcher_client = SearcherServiceClient::connect(searcher_url.to_string()).await?;

        Ok(Self {
            searcher_client,
            access_token: None,
        })
    }

    pub fn set_access_token(&mut self, token: Token) {
        self.access_token = Some(token);
    }

    pub async fn subscribe_mempool(
        &mut self,
        addresses: Vec<String>,
        on_data: impl Fn(proto::dto::MempoolPacket) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut request = tonic::Request::new(MempoolSubscription {
            addresses: if !addresses.is_empty() { Some(AddressSubscriptionV0 { address: addresses }) } else { None },
        });

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(
                    &format!("Bearer {}", access_token.value)
                )?,
            );
        }

        let mut stream = self.searcher_client.subscribe_mempool(request).await?.into_inner();

        tokio::spawn(async move {
            while let Some(response) = stream.message().await.unwrap_or(None) {
                on_data(response);
            }
        });

        Ok(())
    }

    pub async fn send_bundle(&mut self, bundle: proto::dto::Bundle) -> Result<SendBundleResponse, Box<dyn std::error::Error>> {
        let mut request = tonic::Request::new(bundle);

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(
                    &format!("Bearer {}", access_token.value)
                )?,
            );
        }

        let response = self.searcher_client.send_bundle(request).await?;

        Ok(response.into_inner())
    }

    pub async fn get_tip_addresses(&mut self) -> Result<GetTipAddressesResponse, Box<dyn std::error::Error>> {
        let mut request = tonic::Request::new(GetTipAddressesRequest::default());

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(
                    &format!("Bearer {}", access_token.value)
                )?,
            );
        }

        let response = self.searcher_client.get_tip_addresses(request).await?;

        Ok(response.into_inner())
    }
}
