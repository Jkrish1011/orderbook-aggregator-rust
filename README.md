# OrderBook Aggregator

A Rust-based order book aggregator that combines order books from multiple cryptocurrency exchanges.

## Features

- Fetches order book data from two exchanges (Coinbase & Gemini) concurrently
- Aggregates and merges order books
- Calculates best bid/ask prices
- Configurable rate limiting
- Efficient async/await implementation

## Prerequisites

- Rust (latest stable version)
- Cargo

## Installation

```bash
git clone git@github.com:Jkrish1011/orderbook-aggregator-rust.git
cd orderbook-aggregator-rust
cargo build
```


## .env file
```bash
COINBASE_API=https://api.pro.coinbase.com/products/BTC-USD/book?level=2
GEMINI_API=https://api.gemini.com/v1/book/btcusd

```

## Usage

```bash
RUST_LOG=info cargo run -- --qty 189.62521
```