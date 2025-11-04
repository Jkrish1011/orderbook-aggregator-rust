use reqwest::Client;
use serde_json::Value;
use anyhow::Result;

// Getting the raw json as is. Check the coinbase.json and gemini.json for the expected format.
pub async fn get_data(client: &Client, url: String) -> Result<Value> {
    let response = client.get(&url).send().await?;
    let data = response.json::<Value>().await?;
    Ok(data)
}
