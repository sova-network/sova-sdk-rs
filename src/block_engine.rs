use std::str::FromStr;

use tonic::codegen::tokio_stream::Stream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

use crate::proto;
use crate::proto::auth::Token;
use crate::proto::block_engine::block_engine_validator_client::BlockEngineValidatorClient;
use crate::proto::block_engine::SubscribeBundlesRequest;
use crate::proto::dto::MempoolPacket;

pub struct SovaBlockEngine {
    block_engine_client: BlockEngineValidatorClient<Channel>,
    access_token: Option<Token>,
}

impl SovaBlockEngine {
    pub async fn new(
        url: &'static str,
        ca_pem: Option<&str>,
        domain_name: Option<&str>,
        access_token: Token,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let block_engine_client = if let (Some(ca_pem), Some(domain_name)) = (ca_pem, domain_name) {
            let ca = Certificate::from_pem(ca_pem);

            let tls = ClientTlsConfig::new()
                .ca_certificate(ca)
                .domain_name(domain_name);

            let channel = Channel::from_static(url).tls_config(tls)?.connect().await?;

            BlockEngineValidatorClient::new(channel)
        } else {
            BlockEngineValidatorClient::connect(url).await?
        };

        Ok(Self {
            block_engine_client,
            access_token: Some(access_token),
        })
    }

    pub async fn stream_mempool(
        &mut self,
        stream: impl Stream<Item = MempoolPacket> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut request = tonic::Request::new(stream);

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(&format!(
                    "Bearer {}",
                    access_token.value
                ))?,
            );
        }

        self.block_engine_client.stream_mempool(request).await?;

        Ok(())
    }

    pub async fn subscribe_bundles<F>(
        &mut self,
        on_data: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(proto::dto::ValidatorBundle) + Send + 'static,
    {
        let mut request = tonic::Request::new(SubscribeBundlesRequest {});

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(&format!(
                    "Bearer {}",
                    access_token.value
                ))?,
            );
        }

        let mut stream = self
            .block_engine_client
            .subscribe_bundles(request)
            .await?
            .into_inner();

        tokio::spawn(async move {
            while let Some(response) = stream.message().await.unwrap_or(None) {
                on_data(response);
            }
        });

        Ok(())
    }
}
