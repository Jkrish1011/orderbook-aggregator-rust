use clap::{Parser};
use dotenvy::dotenv;
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use serde_json::from_value;
use log::{info, debug};
use env_logger;
use anyhow::Result;
use rust_decimal::Decimal;
use num_format::{Locale, ToFormattedString};

mod helpers;

use helpers::{
    api_client, 
    data_fetcher::get_data, 
    orderbook_merger::{
        merge_sorted_asks,
        merge_sorted_bids,
        calculate_entity_price
    },
    types::{
        CoinbaseResult,
        GeminiResult
    },
    rate_limiter::RateLimiter,
};

use crate::helpers::types::OrderBook;


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
    #[arg(short, long, value_parser = parse_qty, default_value_t = String::from("10.0"))]
    qty: String,
}

fn parse_qty(s: &str) -> Result<String, String> {
    let v: f64 = s.parse::<f64>().map_err(|e| format!("Not a valid quantity {}. Error : {}", s, e))?;

    if !v.is_finite() {
        return Err("Value must be finite".into());
    }

    if v <= 0.0 {
        return Err("Value cannot be negative".into());
    }

    // Not converting to Decimal inorder not to loose precision.
    Ok(s.to_string())
}

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init();
    dotenv().ok();

    let args = Args::parse();
    info!("Orderbook aggregator started");

    let coinbase_api: &str = &env::var("COINBASE_API").unwrap();
    let gemini_api: &str = &env::var("GEMINI_API").unwrap();

    // Create a client to fetch the data from the APIs
    let client = api_client::create_client();

    // Create a rate limiter
    let rate_limiter = Arc::new(RateLimiter::new_per_interval(Duration::from_secs(2)));

    let coinbase_rl = Arc::clone(&rate_limiter);
    let gemini_rl = Arc::clone(&rate_limiter);

    info!("Fetching the Data from Coinbase and Gemini");

    // Fetch the entire dataset from the APIs
    let (result_coinbase, result_gemini) = tokio::join!(
        async {
            coinbase_rl.acquire().await;
            get_data(&client, &coinbase_api).await
        },
        async {
            gemini_rl.acquire().await;
            get_data(&client, &gemini_api).await
        }
    );

    // Parse the data from the APIs
    let coinbase_data: Option<CoinbaseResult> = match result_coinbase {
        Ok(value) => {
            match from_value(value) {
                Ok(data) => Some(data),
                Err(e) => {
                    info!("Error fetching Coinbase data! Error: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            debug!("Error : {:?}", e);
            None
        }
    };
    
    let gemini_data: Option<GeminiResult> = match result_gemini {
        Ok(value) => {
            match from_value(value) {
                Ok(data) => Some(data),
                Err(e) => {
                    info!("Error fetching Gemini data! Error: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            debug!("Error : {:?}", e);
            None
        }
    };

    // If both are None, return an error. Quitting..
    if coinbase_data.is_none() && gemini_data.is_none() {
        return Err(anyhow::anyhow!("Failed to fetch data from Coinbase and Gemini. Quitting..!"));
    }

    // If either is None, use the other one. If both are Some, use both.
    // The logic is designed to move ahead if either of them fails. 
    let coinbase_data = coinbase_data.unwrap_or_default();
    let gemini_data = gemini_data.unwrap_or_default();

    info!("Loaded the data successfully from Coinbase and Gemini");
    info!("Coinbase bids: {}, asks: {}", coinbase_data.bids.len(), coinbase_data.asks.len());
    info!("Gemini bids: {}, asks: {}", gemini_data.bids.len(), gemini_data.asks.len());
    info!("--------------------------------");

    info!("Merging bids");

    // Merge orderbooks 
    let (merged_asks, merged_bids) = tokio::task::spawn_blocking(move || {
        let asks = merge_sorted_asks(coinbase_data.asks, gemini_data.asks);
        let bids = merge_sorted_bids(coinbase_data.bids, gemini_data.bids);
        (asks, bids)
    })
    .await?;

    info!("Asks merged successfully! Total: {}", merged_asks.len());
    info!("Bids merged successfully! Total: {}", merged_bids.len());

    // let cb_first_20 = &merged_asks[..20.min(merged_asks.len())];
    // println!("{:?}", &cb_first_20);

    // let cb_ff = merged_asks.iter().cloned().take(20).collect::<Vec<OrderBook>>();
    // println!("{:?}", cb_ff);
    // let mut index: usize = 0;
    // println!("PRINTING TOP 20 ASKS");
    // for (idx, ob) in merged_asks.iter().enumerate() {
    //     println!("idx : {:?} | Price : {:?} | Exchange : {}", idx, &ob.price, &ob.name);

    //     index+=1;

    //     if index > 20 {
    //         break;
    //     }

    // }

    // index = 0;
    // println!("PRINTING TOP 20 BIDS");
    // for (idx, ob) in merged_bids.iter().enumerate() {
    //     println!("idx : {:?} | Price : {:?} | Exchange: {}", idx, &ob.price, &ob.name);

    //     index+=1;

    //     if index > 20 {
    //         break;
    //     }

    // }

    // for i in [0..20] {
        
    //     println!("Merged Bids :: Index {} : {:?}", i, &merged_bids[i]);
    // }

    // for i in [0..20] {
    //     println!("merged Asks :: Index  : {:?}", &coinbase_data.asks[i]);
    // }

    // for i in [0..20] {
    //     println!("Gemini :: Index  : bid : {:?}", &coinbase_data.bids[i]);
    // }

    // for i in [0..20] {
    //     println!("Gemini :: Index  : asks : {:?}", &coinbase_data.asks[i]);
    // }



    // Calculate prices 
    let qty = Decimal::from_str_exact(&args.qty).unwrap();
    let (buy_price, sell_price) = tokio::task::spawn_blocking(move || {
        let buy = calculate_entity_price(&merged_asks, qty, true, "ASKS"); // asks = ascending
        let sell = calculate_entity_price(&merged_bids, qty, false, "BIDS"); // bids = descending
        (buy, sell)
    })
    .await?;

    println!("--------------------------------");
    info!("Buy Price : {:?}", buy_price);
    info!("Sell Price : {:?}", sell_price);
    let buy_val = buy_price.unwrap().to_string().parse::<f64>().unwrap();
    let sell_val = sell_price.unwrap().to_string().parse::<f64>().unwrap();
    
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


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limiter = Arc::new(RateLimiter::new_per_interval(Duration::from_secs(2)));
        let start = Instant::now();

        // First call should succeed immediately
        rate_limiter.acquire().await;
        let first_call_time = start.elapsed();
        assert!(first_call_time < Duration::from_millis(100), "First call should succeed quickly, took {:?}", first_call_time);

        // Second call should be rate limited and wait ~2 seconds
        let before_second = Instant::now();
        rate_limiter.acquire().await;
        let second_call_wait = before_second.elapsed();
        assert!(second_call_wait >= Duration::from_secs(2) - Duration::from_millis(50), "Second call should wait at least ~2 seconds, waited {:?}", second_call_wait);
        assert!(second_call_wait < Duration::from_secs(3), "Second call should not wait too long, waited {:?}", second_call_wait);

        // Third call should also be rate limited
        let before_third = Instant::now();
        rate_limiter.acquire().await;
        let third_call_wait = before_third.elapsed();
        assert!(third_call_wait >= Duration::from_secs(2) - Duration::from_millis(50), "Third call should wait at least ~2 seconds, waited {:?}", third_call_wait);

        // Total time for 3 calls should be at least 4 seconds (2s between each)
        let total_time = start.elapsed();
        assert!(total_time >= Duration::from_secs(4) - Duration::from_millis(100), "Total time for 3 calls should be at least ~4 seconds, took {:?}", total_time);
    }

    #[tokio::test]
    async fn test_rate_limiter_2() {
        let rate_limiter = Arc::new(RateLimiter::new_per_interval(Duration::from_secs(2)));

        // First call should succeed
        assert!(rate_limiter.try_acquire().await.is_ok(), "First call should succeed");

        // Second call should fail immediately (non-blocking)
        assert!(rate_limiter.try_acquire().await.is_err(), "Second call should fail immediately due to rate limit");

        // After waiting 2 seconds, should succeed again
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(rate_limiter.try_acquire().await.is_ok(), "Call after 2 seconds should succeed");
    }
}