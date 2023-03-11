use anyhow::Result;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct Ip {
    origin: String,
}

struct TestHttpClient;

impl TestHttpClient {
    async fn post() -> Result<Ip> {
        let ip = reqwest::get("http://httpbin.org/ip")
            .await?
            .json::<Ip>()
            .await?;
        Ok(ip)
    }
}

pub async fn test_request_process() -> Result<()> {
    let data = TestHttpClient::post().await?;
    info!(ip = data.origin);
    Ok(())
}
