use clap::{Parser};
use dotenvy::dotenv;
use std::{env};
use serde_json::from_value;
use log::{info, debug};
use env_logger;
use anyhow::Result;

mod helpers;

use helpers::{
    api_client, 
    data_fetcher, 
    types::{
        CoinbaseResult,
        GeminiResult
    }
};


#[derive(Parser, Debug)]
#[command(
    name = "ob-aggregator-rs",
    version = "0.0.1",
    author = "Jayakrishnan <jayakrishnanashok@gmail.com>",
    about = "This app helps you compute the quantity of BTC you can buy or sell",
    long_about = "This is a simple program to analyze the orderbook price and print the best bid and ask price"
)]
struct Args {
    /// Quantity
    #[arg(short, long, value_parser = parse_qty, default_value_t = 10.0)]
    qty: f64,
}

fn parse_qty(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse::<f64>().map_err(|e| format!("Not a valid quantity {}. Error : {}", s, e))?;

    if !v.is_finite() {
        return Err("Value must be finite".into());
    }

    if v < 0.0 {
        return Err("Value cannot be negative".into());
    }
    Ok(v)
}

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init();
    dotenv().ok();

    let args = Args::parse();
    info!("Orderbook aggregator started");

    let coinbase_api = env::var("COINBASE_API").unwrap();
    let gemini_api = env::var("GEMINI_API").unwrap();

    let client = api_client::create_client();

    info!("Fetching the Data from Coinbase and Gemini");

    let (result_coinbase, result_gemini) = tokio::join!(
        data_fetcher::get_data(&client, coinbase_api),
        data_fetcher::get_data(&client, gemini_api),
    );

    let mut coinbase_data: CoinbaseResult = match result_coinbase {
        Ok(value) => {
            info!("Fetched Coinbase data successfull!");
            from_value(value).unwrap()
        },
        Err(e) => {
            debug!("Error : {:?}", e);
            return Err(e.into());
        }
    };
    info!("Total length of bids: {}", coinbase_data.bids.len());
    
    let mut gemini_data: GeminiResult = match result_gemini {
        Ok(value) => {
            info!("Fetched Gemini data successfull!");
            from_value(value).unwrap()
        },
        Err(e) => {
            debug!("Error : {:?}", e);
            return Err(e.into());
        }
    };

    info!("Total length of bids: {}", gemini_data.bids.len());

    Ok(())
}