// Use reqwest to make HTTP requests
use reqwest::Client;
use std::time::Duration;

// Create a client to fetch the data from the APIs
pub fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}