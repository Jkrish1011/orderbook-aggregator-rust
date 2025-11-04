use serde::Deserialize;
use serde::de::{self, Deserializer, SeqAccess, Visitor, Error};
use std::fmt;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct CoinbaseResult {
    pub bids: Vec<CoinbaseOrder>,
    pub asks: Vec<CoinbaseOrder>,
    pub sequence: u64,
    pub auction_mode: bool,
    pub auction: Option<serde_json::Value>,
    pub time: String
}

impl Default for CoinbaseResult {
    fn default() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            sequence: 0,
            auction_mode: false,
            auction: None,
            time: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct CoinbaseOrder {
    pub price: Decimal,
    pub size: Decimal,
    pub num_orders: u64,
}

impl<'de> Deserialize<'de> for CoinbaseOrder {
    fn deserialize<D>(deserializer: D) -> Result<CoinbaseOrder, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Private struct to handle the deserialization logic!
        struct OrderVisitor;

        impl<'de> Visitor<'de> for OrderVisitor {
            // Target type for deserialized value
            type Value = CoinbaseOrder;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                // To present an error file deserialization fails!
                write!(f, "an array like [\"price\",\"size\",num_orders]")
            }

            // Handles the deserialization of a sequence (an array) into a CoinbaseOrder
            fn visit_seq<A>(self, mut seq: A) -> Result<CoinbaseOrder, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Extracts the elements
                let price_str: String = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let size_str: String = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?;
                let num_orders: u64 = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(2, &self))?;

                // Converts the extracted elements to the target type
                let price = Decimal::from_str(&price_str).map_err(Error::custom)?;
                let size = Decimal::from_str(&size_str).map_err(Error::custom)?;
    
                Ok(CoinbaseOrder { price, size, num_orders })
            }
        }

        deserializer.deserialize_seq(OrderVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct GeminiResult {
    pub bids: Vec<GeminiOrder>,
    pub asks: Vec<GeminiOrder>
}

impl Default for GeminiResult {
    fn default() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GeminiOrder {
    #[serde(deserialize_with = "from_str_to_decimal")]
    pub price: Decimal,

    #[serde(deserialize_with = "from_str_to_decimal")]
    pub amount: Decimal,

    #[serde(deserialize_with = "from_str_to_u64")]
    pub timestamp: u64 
}

// Taking a deserializer D that should implement the Deserializer trait
fn from_str_to_decimal<'de, D>(d: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    // Return a Result which is Decimal or the deserialization error
    Decimal::from_str(&s).map_err(Error::custom)
}

// Taking a deserializer D that should implement the Deserializer trait.
fn from_str_to_f64<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    // Return a Result which is f64 or the deserialization error
    s.parse::<f64>().map_err(Error::custom)
}

// Taking a deserializer D that should implement the Deserializer trait.
fn from_str_to_u64<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    // Return a Result which is u64 or the deserialization error
    s.parse::<u64>().map_err(Error::custom)
}

// Orderbook for Merged data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    pub price: Decimal,
    pub size: Decimal,
}

// Implementing PartialOrd for OrderBook
impl PartialOrd for OrderBook {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Implementing Ord for OrderBook
impl Ord for OrderBook {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.cmp(&other.price)
    }
}
