use clap::{Parser};
use dotenvy::dotenv;
use std::{env};
use serde_json::from_value;
use log::{info, debug};
use env_logger;
use anyhow::Result;
use rust_decimal::Decimal;
use num_format::{Locale, ToFormattedString};

mod helpers;

use helpers::{
    api_client, 
    data_fetcher, 
    orderbook_merger,
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

    info!("Loaded the data successfully from Coinbase and Gemini");
    info!("Coinbase bids: {}, asks: {}", coinbase_data.bids.len(), coinbase_data.asks.len());
    info!("Gemini bids: {}, asks: {}", gemini_data.bids.len(), gemini_data.asks.len());
    info!("--------------------------------");

    info!("Merging bids");

        // Merge orderbooks concurrently
    let (merged_asks, merged_bids) = tokio::task::spawn_blocking(move || {
        let asks = orderbook_merger::merge_sorted_asks(coinbase_data.asks, gemini_data.asks);
        let bids = orderbook_merger::merge_sorted_bids(coinbase_data.bids, gemini_data.bids);
        (asks, bids)
    })
    .await?;

    info!("Asks merged successfully! Total: {}", merged_asks.len());
    info!("Bids merged successfully! Total: {}", merged_bids.len());

    // Calculate prices concurrently
    let qty = Decimal::from_f64_retain(args.qty).unwrap();
    let (buy_price, sell_price) = tokio::task::spawn_blocking(move || {
        let buy = orderbook_merger::calculate_entity_price(&merged_bids, qty);
        let sell = orderbook_merger::calculate_entity_price(&merged_asks, qty);
        (buy, sell)
    })
    .await?;

    println!("--------------------------------");
    let buy_val = buy_price.to_string().parse::<f64>().unwrap();
    let sell_val = sell_price.to_string().parse::<f64>().unwrap();
    
    // Format with commas by converting to cents (integer), formatting, then adding decimal
    let buy_cents = (buy_val * 100.0).round() as i64;
    let sell_cents = (sell_val * 100.0).round() as i64;
    
    println!("To buy {} BTC: ${}.{:02}", args.qty, 
        (buy_cents / 100).to_formatted_string(&Locale::en), 
        buy_cents.abs() % 100);
    println!("To sell {} BTC: ${}.{:02}", args.qty, 
        (sell_cents / 100).to_formatted_string(&Locale::en), 
        sell_cents.abs() % 100);
        
    Ok(())
}