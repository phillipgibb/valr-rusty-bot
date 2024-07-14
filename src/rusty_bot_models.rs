#![allow(dead_code)]
#![allow(unused_variables)]
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};

#[derive(Deserialize, Debug)]
#[serde(tag="type")]
pub enum WsMessage {
    #[serde(rename = "BALANCE_UPDATE", deserialize_with = "ws_deserializer")]
    BalanceUpdate(Box<BalanceUpdate>),
    #[serde(rename = "OPEN_ORDERS_UPDATE", deserialize_with = "ws_deserializer")]
    OpenOrdersUpdate(Vec<Order>),
    #[serde(rename = "NEW_TRADE_BUCKET", deserialize_with = "ws_deserializer")]
    NewTradeBucket(Box<TradePriceBucketUpdate>),
    #[serde(rename = "OB_L1_D1_SNAPSHOT", deserialize_with = "ws_deserializer")]
    OrderbookLvOneDepthOneSnapshot(Box<DepthOrderBookSnapshot>),
    #[serde(rename = "OB_L1_D10_SNAPSHOT", deserialize_with = "ws_deserializer")]
    OrderbookLvOneDepthTenSnapshot(Box<DepthOrderBookSnapshot>),
    #[serde(rename = "AUTHENTICATED")]
    Authenticated,
    #[serde(rename = "SUBSCRIBED")]
    Subscribed,
    #[serde(rename = "PONG")]
    Pong,
    #[serde(rename = "UNSUPPORTED")]
    Unsupported
}

fn ws_deserializer<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct WsMessage<T> {
        #[serde(alias = "d")]
        data: T
    }
    let d = WsMessage::deserialize(deserializer)?.data;
    Ok(d)
}

#[serde_as]
#[derive(Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct MarkPriceBucket {
    #[serde(rename = "currencyPairSymbol")]
    pub currency_pair_symbol: String,
    #[serde(rename = "bucketPeriodInSeconds")]
    pub bucket_period_in_seconds: u16,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde_as(as = "DisplayFromStr")]
    pub open: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub close: f64,
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

#[derive(Deserialize, Debug, Clone)]
pub struct CurrencyPair {
    pub symbol: String,
    #[serde(rename = "baseCurrency")]
    pub base_currency: String,
    #[serde(rename = "quoteCurrency")]
    pub quote_currency: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    pub active: bool,
    #[serde(rename = "minBaseAmount")]
    min_base_amount: String,
    #[serde(rename = "maxBaseAmount")]
    max_base_amount: String,
    #[serde(rename = "minQuoteAmount")]
    min_quote_amount: String,
    #[serde(rename = "maxQuoteAmount")]
    max_quote_amount: String,
    #[serde(rename = "tickSize")]
    tick_size: String,
    #[serde(rename = "baseDecimalPlaces")]
    base_decimal_places:String,
    #[serde(rename = "marginTradingAllowed")]
    margin_trading_allowed: bool,
    #[serde(rename = "currencyPairType")]
    currency_pair_type: String,
    #[serde(rename = "initialMarginFraction")]
    initial_margin_fraction: Option<String>,
    #[serde(rename = "maintenanceMarginFraction")]
    maintenance_margin_fraction: Option<String>,
    #[serde(rename = "autoCloseMarginFraction")]
    auto_close_margin_fraction: Option<String>
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

#[serde_as]
#[derive(Deserialize, Debug, PartialEq, PartialOrd)]
pub struct TradePriceBucketUpdate {
    #[serde(rename = "currencyPairSymbol")]
    pub currency_pair_symbol: String,
    #[serde(rename = "bucketPeriodInSeconds")]
    pub bucket_period_in_seconds: u16,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde_as(as = "DisplayFromStr")]
    pub open: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub close: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub volume: f64,
    #[serde(rename = "quoteVolume")]
    #[serde_as(as = "DisplayFromStr")]
    pub quote_volume: f64,
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
