pub use cloudflare::endpoints::dns::DnsContent;
use cloudflare::{
    endpoints::dns::{
        DnsRecord, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams,
    },
    framework::response::ApiFailure,
};

use once_cell::sync::Lazy;

fn cloudflare_client(
    token: &str,
) -> Result<cloudflare::framework::async_api::Client, anyhow::Error> {
    use cloudflare::framework::{auth::Credentials, Environment, HttpApiClientConfig};
    cloudflare::framework::async_api::Client::new(
        Credentials::UserAuthToken {
            token: token.to_string(),
        },
        HttpApiClientConfig::default(),
        Environment::Production,
    )
}

static CLOUDFLARE_CLIENT: Lazy<cloudflare::framework::async_api::Client> =
    Lazy::new(|| cloudflare_client(include_str!("../config/cloudflare-token.txt")).unwrap());

pub struct ListDnsRecordsBuilder<'a> {
    inner: ListDnsRecords<'a>,
}
impl<'a> ListDnsRecordsBuilder<'a> {
    pub fn new(zone_id: &'a str) -> Self {
        Self {
            inner: ListDnsRecords {
                zone_identifier: zone_id,
                params: ListDnsRecordsParams::default(),
            },
        }
    }
    pub fn name(mut self, name: String) -> Self {
        self.inner.params.name = Some(name);
        self
    }
    pub async fn execute(self) -> Result<Vec<DnsRecord>, ApiFailure> {
        let res = CLOUDFLARE_CLIENT.request_handle(&self.inner).await?;
        Ok(res.result)
    }
}

pub struct UpdateDnsRecordBuilder<'a> {
    inner: UpdateDnsRecord<'a>,
}
impl<'a> UpdateDnsRecordBuilder<'a> {
    pub fn new(record: &'a DnsRecord) -> Self {
        Self {
            inner: UpdateDnsRecord {
                zone_identifier: &record.zone_id,
                identifier: &record.id,
                params: UpdateDnsRecordParams {
                    ttl: Some(record.ttl),
                    proxied: Some(record.proxied),
                    name: &record.name,
                    content: record.content.clone(),
                },
            },
        }
    }
    pub fn content(mut self, content: DnsContent) -> Self {
        self.inner.params.content = content;
        self
    }
    pub async fn execute(self) -> Result<DnsRecord, ApiFailure> {
        let res = CLOUDFLARE_CLIENT.request_handle(&self.inner).await?;
        Ok(res.result)
    }
}
