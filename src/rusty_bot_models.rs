#![allow(dead_code)]
#![allow(unused_variables)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct MarkPriceBucket {
    #[serde(rename = "currencyPairSymbol")]
    pub currency_pair_symbol: String,
    #[serde(rename = "bucketPeriodInSeconds")]
    pub bucket_period_in_seconds: u16,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde_as(as = "DisplayFromStr")]
    pub open: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub high: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub low: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub close: i32,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketMessage {
    pub r#type: String,
    #[serde(rename = "currencyPairSymbol")]
    #[serde(alias = "ps")]
    pub pair: Option<String>,
    #[serde(alias = "d")]
    pub data: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    pub symbol: String,
    #[serde(rename = "decimalPlaces")]
    pub decimal_places: u8,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "shortName")]
    pub short_name: String,
    #[serde(rename = "longName")]
    pub long_name: String,
    #[serde(rename = "supportedWithdrawDecimalPlaces")]
    pub supported_withdraw_decimal_places: u8,
    pub collateral: bool,
    #[serde(rename = "collateralWeight")]
    pub collateral_weight: String,
}

#[derive(Deserialize, Debug)]
pub struct BalanceUpdate {
    pub currency: Currency,
    pub available: String,
    pub reserved: String,
    pub total: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "lendReserved")]
    pub lend_reserved: String,
    #[serde(rename = "borrowReserved")]
    pub borrow_reserved: Option<String>,
    #[serde(rename = "borrowedAmount")]
    pub borrowed_amount: String,
    #[serde(rename = "totalInReference")]
    pub total_in_reference: String,
    #[serde(rename = "totalInReferenceWeighted")]
    pub total_in_reference_weighted: String,
    #[serde(rename = "referenceCurrency")]
    pub reference_currency: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderBookEntry {
    #[serde(rename = "currencyPair")]
    pub currency_pair: String,
    #[serde(rename = "orderCount")]
    pub order_count: i16,
    pub price: String,
    pub quantity: String,
    pub side: String,
}

#[derive(Deserialize, Debug)]
pub struct DepthOrderBookSnapshot {
    #[serde(rename = "Asks")]
    #[serde(alias = "a")]
    pub asks: Vec<Vec<String>>,
    #[serde(rename = "Bids")]
    #[serde(alias = "b")]
    pub bids: Vec<Vec<String>>,
    #[serde(rename = "lastChange")]
    #[serde(alias = "lc")]
    pub last_change: i64,
}

#[derive(Deserialize, Debug)]
pub struct AggregatedOrderBookUpdate {
    #[serde(rename = "Asks")]
    #[serde(alias = "a")]
    pub asks: Vec<OrderBookEntry>,
    #[serde(rename = "Bids")]
    #[serde(alias = "b")]
    pub bids: Vec<OrderBookEntry>,
}

#[derive(Deserialize, Debug)]
pub struct TradePriceBucketUpdate {
    #[serde(rename = "currencyPairSymbol")]
    pub currency_pair_symbol: String,
    #[serde(rename = "bucketPeriodInSeconds")]
    pub bucket_period_in_seconds: u16,
    #[serde(rename = "startTime")]
    pub start_time: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderLevelOrder {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub quantity: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderLevel {
    #[serde(rename = "Price")]
    pub price: String,
    #[serde(rename = "Orders")]
    pub orders: Vec<OrderLevelOrder>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookUpdate {
    // r#type: String,
    #[serde(rename = "currency_pair_symbol")]
    pub pair: String,
    pub data: OrderBookData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookData {
    #[serde(rename = "LastChange")]
    pub last_change: u64,
    #[serde(rename = "Asks")]
    pub asks: Vec<OrderLevel>,
    #[serde(rename = "Bids")]
    pub bids: Vec<OrderLevel>,
    #[serde(rename = "SequenceNumber")]
    pub sn: u64,
    #[serde(rename = "Checksum")]
    pub checksum: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub side: String,
    #[serde(rename = "remainingQuantity")]
    pub remaining_quantity: Option<String>,
    pub price: String,
    #[serde(rename = "currencyPair")]
    pub currency_pair: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "originalQuantity")]
    pub original_quantity: String,
    #[serde(rename = "filledPercentage")]
    pub filled_percentage: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub status: String,
    pub r#type: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "allowMargin")]
    pub allow_margin: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubAccountResponse {
    pub id: String,
}
