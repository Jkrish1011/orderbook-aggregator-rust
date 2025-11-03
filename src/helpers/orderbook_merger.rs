use rust_decimal::Decimal;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::helpers::types::{CoinbaseOrder, GeminiOrder, OrderBook};

// Merge sorted asks from both coinbase and gemini. Ascending Order
// Using iterator for efficiency here. Not collecting here.
pub fn merge_sorted_asks(coinbase_asks: Vec<CoinbaseOrder>,gemini_asks: Vec<GeminiOrder>) -> Vec<OrderBook> {
    let mut merged: Vec<OrderBook> = Vec::with_capacity(coinbase_asks.len() + gemini_asks.len());

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

pub fn calculate_entity_price(entity: &[OrderBook], quantity: Decimal) -> Decimal {
    let mut total_cost = Decimal::ZERO;
    let mut remaining_quantity = quantity;

    for entry in entity.iter() {
        if remaining_quantity < entry.size {
            total_cost += entry.price * remaining_quantity;
            break;
        } else {
            total_cost += entry.price * entry.size;
            remaining_quantity -= entry.size;
        }
    }

    total_cost
}



