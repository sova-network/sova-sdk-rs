use std::str::FromStr;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

use crate::proto;

use crate::proto::auth::Token;
use crate::proto::searcher::searcher_service_client::SearcherServiceClient;
use crate::proto::searcher::{AddressSubscriptionV0, GetTipAddressesRequest, GetTipAddressesResponse, MempoolSubscription, SendBundleResponse};

pub struct MevtonSearcher {
    searcher_client: SearcherServiceClient<Channel>,
    access_token: Option<Token>,
}

impl MevtonSearcher {
    pub async fn new(url: &'static str, ca_pem: Option<&str>, domain_name: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let searcher_client = if let (Some(ca_pem), Some(domain_name)) = (ca_pem, domain_name) {
            let ca = Certificate::from_pem(ca_pem);

            let tls = ClientTlsConfig::new()
                .ca_certificate(ca)
                .domain_name(domain_name);

            let channel = Channel::from_static(url)
                .tls_config(tls)?
                .connect()
                .await?;

            SearcherServiceClient::new(channel)
        } else {
            SearcherServiceClient::connect(url).await?
        };

        Ok(Self {
            searcher_client,
            access_token: None,
        })
    }

    pub fn set_access_token(&mut self, token: Token) {
        self.access_token = Some(token);
    }

    pub async fn subscribe_mempool<F>(
        &mut self,
        addresses: Vec<String>,
        on_data: F,
    ) -> Result<(), Box<dyn std::error::Error>>
        where
            F: Fn(proto::dto::MempoolPacket) + Send + 'static,
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
