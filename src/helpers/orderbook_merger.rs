use rust_decimal::Decimal;
use crate::helpers::types::{CoinbaseOrder, GeminiOrder, OrderBook};
use log::{info};

// Merge sorted asks from both coinbase and gemini. Ascending Order
// Using iterator for efficiency here. Not collecting here.
pub fn merge_sorted_asks(coinbase_asks: Vec<CoinbaseOrder>,gemini_asks: Vec<GeminiOrder>) -> Vec<OrderBook> {
    let mut merged: Vec<OrderBook> = Vec::with_capacity(coinbase_asks.len() + gemini_asks.len());

    // Ensure inputs are sorted
    let mut coinbase_asks = coinbase_asks;
    let mut gemini_asks = gemini_asks;
    coinbase_asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    gemini_asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    // Then proceed with merge...
    let mut cb_iter = coinbase_asks.into_iter().peekable();
    let mut gem_iter = gemini_asks.into_iter().peekable();

    loop {
        match (cb_iter.peek(), gem_iter.peek()) {
            (Some(cb), Some(gem)) => {
                if cb.price <=  gem.price {
                    let order = cb_iter.next().unwrap();
                    merged.push(OrderBook{
                        price: order.price,
                        size: order.size
                    });
                } else {
                    let order = gem_iter.next().unwrap();
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.amount,
                    });
                }
                
            }
            (Some(_), None) => {
                // Ony coinbase left
                for order in cb_iter {
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.size,
                    });
                }
                break;
            }
            (None, Some(_)) => {
                // Only Gemini Left
                for order in gem_iter {
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.amount,
                    });
                }
                break;
            }
            (None, None) => {
                break;
            }
        }
    }
    merged
}

// Merging sorted bids from Coinbase and Gemini. Descending price order.
pub fn merge_sorted_bids(coinbase_bids: Vec<CoinbaseOrder>, gemini_bids: Vec<GeminiOrder>) -> Vec<OrderBook> {
    let mut merged = Vec::with_capacity(coinbase_bids.len() + gemini_bids.len());
    
    // Ensure inputs are sorted (descending)
    let mut coinbase_bids = coinbase_bids;
    let mut gemini_bids = gemini_bids;
    coinbase_bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()); // Descending
    gemini_bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());   // Descending

// Then proceed with merge...

    let mut cb_iter = coinbase_bids.into_iter().peekable();
    let mut gem_iter = gemini_bids.into_iter().peekable();

    loop {
        match (cb_iter.peek(), gem_iter.peek()) {
            (Some(cb), Some(gem)) => {
                if cb.price >= gem.price {
                    let order = cb_iter.next().unwrap();
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.size,
                    });
                } else {
                    let order = gem_iter.next().unwrap();
                    merged.push(OrderBook{
                        price: order.price,
                        size: order.amount,
                    });
                }
                
            }
            (Some(_), None) => {
                // Only coinbase left
                for order in cb_iter {
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.size,
                    });
                }
                break;
            }
            (None, Some(_)) => {
                // Only gemini order left.
                for order in gem_iter {
                    merged.push(OrderBook {
                        price: order.price,
                        size: order.amount,
                    });
                }
                break;
            }
            (None, None) => {
                break;
            }
        }
    }
    merged
}

pub fn calculate_entity_price(entity: &[OrderBook], quantity: Decimal, is_ascending: bool, order_type: &str) -> Result<Decimal, String> {
    let mut total_cost = Decimal::ZERO;
    let mut remaining_quantity = quantity;
    let original_quantity = quantity;
    let mut count = 0;
    let mut total_size_available = Decimal::ZERO;
    let mut tiny_orders = 0;

    // Insignificant here. But just calculating very Tiny orders to identify any bugs of any sort.
    for entry in entity.iter() {
        total_size_available += entry.size;
        // To check if BTC size is < 0.0001
        if entry.size < Decimal::new(1, 4) {
            tiny_orders += 1;
        }
    }

    info!("[{}] Total Quantity Available is : {}", order_type, total_size_available);
    info!("[{}] Total tiny orders: {}", order_type, tiny_orders);

    // Checking if all orders are sorted correctly!
    if entity.len() > 1 {
        // Verify ordering (for asks: ascending, for bids: descending)
        let mut is_sorted = true;
        for i in 1..entity.len() {

            let is_wrong_order = if is_ascending {
                entity[i-1].price > entity[i].price // Should be ascending
            } else {
                entity[i-1].price < entity[i].price // Should be descending
            };

            if is_wrong_order {
                is_sorted = false;
                info!("WARNING: Orders not sorted! Order {} (price {}) vs Order {} (price {})", i-1, entity[i-1].price, i, entity[i].price);
                break;
            }
        }

        if !is_sorted {
            info!("WARNING: Order book is not properly sorted!");
        }
    }


    for entry in entity.iter() {

        if entry.size == Decimal::ZERO {
            info!("WARNING: Order at price {} has ZERO size!", entry.price);
            continue;
        }

        if remaining_quantity <= entry.size {
            // partial fill of the given order quantity
            total_cost += entry.price * remaining_quantity;
            count += 1;
            remaining_quantity = Decimal::ZERO; // To tackle the wrong firing of Insufficient Liquidity error.
            break;
        } else {
            total_cost += entry.price * entry.size;
            remaining_quantity -= entry.size;
            count += 1;

            if remaining_quantity <= Decimal::ZERO {
                info!("WARNING: remaining_quantity became negative: {}", remaining_quantity);
                remaining_quantity = Decimal::ZERO;
                break; // Quit and avoid further processing!
            }
        }
    }

    info!("Total orders processed: {}", count);
    info!("Remaining quantity after processing: {}", remaining_quantity);

   
    if remaining_quantity > Decimal::ZERO {
        info!("Insufficient liquidity: requested {}, only {} available", original_quantity, original_quantity - remaining_quantity);
    }

    Ok(total_cost)
}



