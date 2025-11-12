use reqwest::Client;
use serde_json::Value;
use anyhow::{Result, bail};
use std::time::Duration;

/*
    Taking parameters as &str is more memory efficient and doesn't require ownership movement.
*/
pub async fn get_data(client: &Client, url: &str) -> Result<Value> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(60))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        bail!("Request failed: {} - {}", status, text);
    }

    let data = response.json::<Value>().await?;
    Ok(data)
}