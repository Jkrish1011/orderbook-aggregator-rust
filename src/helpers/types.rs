
use serde::Deserialize;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct CoinbaseResult {
    pub bids: Vec<CoinbaseOrder>,
    pub asks: Vec<CoinbaseOrder>,
    pub sequence: u64,
    pub auction_mode: bool,
    pub auction: Option<serde_json::Value>,
    pub time: String
}

#[derive(Debug)]
pub struct CoinbaseOrder {
    pub price: f64,
    pub size: f64,
    pub num_orders: u64,
}

impl<'de> Deserialize<'de> for CoinbaseOrder {
    fn deserialize<D>(deserializer: D) -> Result<CoinbaseOrder, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderVisitor;

        impl<'de> Visitor<'de> for OrderVisitor {
            type Value = CoinbaseOrder;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "an array like [\"price\",\"size\",num_orders]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<CoinbaseOrder, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let price_str: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let size_str: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let num_orders: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                let price = price_str.parse::<f64>().map_err(de::Error::custom)?;
                let size = size_str.parse::<f64>().map_err(de::Error::custom)?;

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

#[derive(Debug, Deserialize)]
pub struct GeminiOrder {
    #[serde(deserialize_with = "from_str_to_f64")]
    pub price: f64,

    #[serde(deserialize_with = "from_str_to_f64")]
    pub amount: f64,

    #[serde(deserialize_with = "from_str_to_u64")]
    pub timestamp: u64 
}

fn from_str_to_f64<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse::<f64>().map_err(de::Error::custom)
}

fn from_str_to_u64<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse::<u64>().map_err(de::Error::custom)
}