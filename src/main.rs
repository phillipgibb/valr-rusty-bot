#![allow(unused_variables)]

mod config;
mod rusty_bot_models;
mod strategies;
mod tests;

use crate::config::{ConfigProvider, DotEnvConfigProvider};
use crate::rusty_bot_models::WsMessage;
use crate::strategies::break_of_structure::helper::{
    create_http_request, create_ws_request, execute_strategy,
};
use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use convert_case::{Case, Casing};
use futures_util::future::try_join_all;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use lazy_static::lazy_static;
use log::{error, warn};
use rusty_bot_models::{
    AggregatedOrderBookUpdate, BalanceUpdate, DepthOrderBookSnapshot, MarkPriceBucket, Order,
    OrderBookData, TradePriceBucketUpdate,
};
use serde_json::json;
use std::mem::replace;
use std::str::FromStr;
use std::string::String;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::http;

const FIVE_MINUTE_BUCKET_SECONDS: &str = "300";

lazy_static! {
    static ref BUCKET_PRICES: Arc<RwLock<Vec<MarkPriceBucket>>> = Arc::new(RwLock::new(vec![]));
    static ref BIDS: Arc<RwLock<Vec<Vec<String>>>> = Arc::new(RwLock::new(vec![]));
    static ref ASKS: Arc<RwLock<Vec<Vec<String>>>> = Arc::new(RwLock::new(vec![]));
    static ref ORDERS: Arc<RwLock<Vec<Order>>> = Arc::new(RwLock::new(vec![]));
}

#[tokio::main]
async fn main() {
    println!("Hello, VALR Rusty Trader!");
    let env_config_provider = DotEnvConfigProvider::new();
    let config = env_config_provider.get_config();
    env_logger::init();
    let current_date_time = Utc::now().naive_utc();
    let one_hour_ago_date_time = current_date_time - Duration::hours(1);
    get_historical_sixty_second_mark_price_buckets_for_pair(
        &config.market,
        one_hour_ago_date_time.to_string(),
        current_date_time.to_string(),
    )
    .await
    .expect("Error getting historical mark price buckets");
    // get_open_orders_for_pair(&config.api_key, &config.api_secret, &config.market).await.expect("Error getting open orders");

    let mut handles = vec![];
    let mut trade_update_read_handles = subscribe_to_trade_updates(
        &config.api_key,
        &config.api_secret,
        &config.market,
        config.strategy.clone(),
    )
    .await;
    let mut account_handlers =
        subscribe_to_account_updates(&config.api_key, &config.api_secret, config.strategy.clone())
            .await;
    handles.append(&mut trade_update_read_handles);
    handles.append(&mut account_handlers);

    try_join_all(handles.into_iter())
        .await
        .expect("Failure joining tasks");
}

async fn subscribe_to_account_updates(
    api_key: &str,
    api_secret: &str,
    strategy: String,
) -> Vec<JoinHandle<()>> {
    let url = Uri::from_str("wss://api.valr.com/ws/account");

    let request = create_ws_request(
        url.unwrap(),
        api_key,
        api_secret,
        "/ws/account",
        "GET",
        None,
    );
    let (ws_stream, _) = connect_async(request)
        .await
        .expect("Error connecting to Account WebSocket");
    let (write, read) = ws_stream.split();

    let account_handle = tokio::spawn(handle_ws_incoming_messages(read, strategy, "account"));
    let ping_handle = create_ping_thread(write, Utc::now(), String::from("Account WS"));

    vec![account_handle, ping_handle]
}

async fn subscribe_to_trade_updates(
    api_key: &str,
    api_secret: &str,
    pair_symbol: &str,
    strategy: String,
) -> Vec<JoinHandle<()>> {
    let url = Uri::from_str("wss://api.valr.com/ws/trade");
    let message = json!(
        {
        "type": "SUBSCRIBE",
        "subscriptions": [
            {
                "event": "NEW_TRADE_BUCKET",
                "pairs": [format!("{}", pair_symbol)]
            },
            {
                "event": "OB_L1_D10_SNAPSHOT",
                "pairs": [format!("{}", pair_symbol)]
            },
            {
                "event": "NEW_TRADE"
            },
            {
                "event": "ORDER_STATUS_UPDATE"
            }
            // {
            //     "event": "FULL_ORDERBOOK_UPDATE",
            //     "pairs": [format!("{}", pair_symbol)]
            // },
            // {
            //     "event": "AGGREGATED_ORDERBOOK_UPDATE",
            //     "pairs": [format!("{}", pair_symbol)]
            // },

        ]
    });

    let request = create_ws_request(url.unwrap(), api_key, api_secret, "/ws/trade", "GET", None);
    let (ws_stream, _) = connect_async(request)
        .await
        .expect("Error connecting to Trade WebSocket");

    let (mut write, read) = ws_stream.split();
    let subscribe_handle = tokio::spawn(handle_ws_incoming_messages(read, strategy, "trade"));

    write
        .send(Message::from(message.to_string()))
        .await
        .expect("Failed to send message");

    let ping_handle = create_ping_thread(write, Utc::now(), String::from("Trade   WS"));
    vec![subscribe_handle, ping_handle]
}

fn create_ping_thread(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut current_time: DateTime<Utc>,
    name: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let time_delta = now.signed_duration_since(current_time);

            if time_delta.num_seconds() >= 10 {
                current_time = Utc::now();
                let ping_message = json!({
                    "type": "PING"
                });

                match write
                    .send(Message::from(ping_message.to_string()))
                    .await
                {
                    Ok(i) => {
                        println!(
                            "{}| {} Ping {}",
                            current_time.to_rfc3339().blue(),
                            name.bold().green(),
                            "Sent".yellow()
                        );
                    }
                    Err(e) => {
                        warn!("Error sending ping: {}", e.to_string())
                    }
                }
            };
        }
    })
}

async fn handle_ws_incoming_messages(
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    strategy: String,
    subscription_type: &str,
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let ws_message = serde_json::from_str::<WsMessage>(text.as_str());
                let current_time = Utc::now();
                match ws_message {
                    Ok(serialized) => match serialized {
                        WsMessage::BalanceUpdate(balance_update) => {
                            handle_balance_update(*balance_update)
                        }
                        WsMessage::OpenOrdersUpdate(order_update) => {
                            handle_order_update(order_update)
                        }
                        WsMessage::NewTradeBucket(trade_price_bucket_update) => {
                            handle_trade_price_bucket_update(
                                *trade_price_bucket_update,
                                strategy.clone(),
                            )
                            .await;
                        }
                        WsMessage::OrderbookLvOneDepthOneSnapshot(ob) => {}
                        WsMessage::OrderbookLvOneDepthTenSnapshot(ob) => {}
                        WsMessage::Subscribed => {
                            println!("{}| Subscribed {}", current_time.to_rfc3339().blue(), text.green())
                        }
                        WsMessage::Authenticated => {
                            println!("{}| Authenticated", current_time.to_rfc3339().blue())
                        }
                        WsMessage::Unsupported => {
                            println!(
                                "{}| Unsupported {} {}",
                                current_time.to_rfc3339().blue(),
                                subscription_type,
                                text
                            )
                        }
                        WsMessage::Pong => {
                            println!(
                                "{}| {} {} message Pong",
                                current_time.to_rfc3339().blue(),
                                subscription_type.to_case(Case::Snake).green(),
                                "WS".green()
                            )
                        }
                    },
                    Err(e) => {
                        let current_time = Utc::now();
                        println!(
                            "{}| {} {} message {}",
                            current_time.to_rfc3339().blue(),
                            subscription_type.to_case(Case::Snake).green(),
                            "WS".green(),
                            e.to_string().red()
                        )
                    }
                }
            }
            Ok(t) => error!(
                "Unexpected Message during the WebSocket communication: {}",
                t
            ), // Ignore non-Text messages
            Err(e) => error!("Error during the Trade WebSocket communication: {}", e),
        }
    }
}

fn handle_order_update(order: Vec<Order>) {
    println!();
    println!("OPEN ORDERS UPDATE: {:?}", order);
    println!();
}

async fn handle_orderbook_level_1_depth_1_snapshot_update(orderbook_data: DepthOrderBookSnapshot) {
    // Access parsed fields from the struct
    // println!("Last Change: {}", orderbook_data.last_change);
    // Iterate over Asks and Bids
    let mut asks_writer = ASKS.write().await;
    let mut bids_writer = BIDS.write().await;
    asks_writer.clear();
    bids_writer.clear();

    for ask in orderbook_data.asks {
        // println!("Ask Price: {}, quantity: {}", ask[0], ask[1]);
        asks_writer.push(ask);
    }

    for bid in orderbook_data.bids {
        // println!("Bid Price: {}, quantity: {}", bid[0], bid[1]);
        bids_writer.push(bid);
    }
    drop(asks_writer);
    drop(bids_writer);
}

async fn handle_trade_price_bucket_update(
    trade_price_bucket_update: TradePriceBucketUpdate,
    strategy: String,
) {
    if trade_price_bucket_update.bucket_period_in_seconds != 60 {
        //300
        return;
    }
    let up_arrow: String = String::from_utf16(&[0x2B06]).unwrap();
    let down_arrow: String = String::from_utf16(&[0x2B07]).unwrap();
    let circle: String = String::from_utf16(&[0x23FA]).unwrap();

    let c_lock = BUCKET_PRICES.clone();

    tokio::spawn(async move {
        // While main has an active read lock, we acquire one too.
        let bpr = BUCKET_PRICES.read().await;
        let position = {
            bpr.iter()
                .position(|b| b.start_time == trade_price_bucket_update.start_time)
        };

        let last_position = bpr.iter().last().unwrap();

        let close_direction = if last_position.close < trade_price_bucket_update.close {
            up_arrow.green()
        } else if last_position.close > trade_price_bucket_update.close {
            down_arrow.red()
        } else {
            circle.white()
        };

        let high_direction = if last_position.high < trade_price_bucket_update.high {
            up_arrow.green()
        } else if last_position.high > trade_price_bucket_update.high {
            down_arrow.red()
        } else {
            circle.white()
        };

        let low_direction = if last_position.low < trade_price_bucket_update.low {
            up_arrow.green()
        } else if last_position.low > trade_price_bucket_update.low {
            down_arrow.red()
        } else {
            circle.white()
        };

        println!(
            "{} for {} received. CLOSE: {}{} , HIGH: {}{} , LOW: {}{} , start_time: {}",
            "Trade Price Bucket Update".on_bright_blue(),
            trade_price_bucket_update.currency_pair_symbol.green(),
            trade_price_bucket_update.close.to_string().yellow(),
            close_direction,
            trade_price_bucket_update.high.to_string().yellow(),
            high_direction,
            trade_price_bucket_update.low.to_string().yellow(),
            low_direction,
            trade_price_bucket_update.start_time.blue()
        );

        drop(bpr);
        let mpb = create_mark_price_bucket(trade_price_bucket_update);
        let mut bpw = BUCKET_PRICES.write().await;

        match position {
            None => {
                bpw.push(mpb);
                drop(bpw);
            }
            Some(_) => {
                let _ = replace(&mut bpw[position.unwrap()], mpb);
            }
        }
    })
    .await
    .expect("The spawned task has panicked");
    tokio::spawn(async move {
        // While main has an active read lock, we acquire one too.
        // let bpr = &c_lock.read().await;
        let bpr = BUCKET_PRICES.read().await;

        execute_strategy(strategy.clone().as_str(), bpr.to_vec(), &ASKS, &BIDS).await;
    });
}

fn create_mark_price_bucket(trade_price_bucket_update: TradePriceBucketUpdate) -> MarkPriceBucket {
    MarkPriceBucket {
        currency_pair_symbol: trade_price_bucket_update.currency_pair_symbol,
        bucket_period_in_seconds: trade_price_bucket_update.bucket_period_in_seconds,
        start_time: trade_price_bucket_update.start_time,
        open: trade_price_bucket_update.open,
        high: trade_price_bucket_update.high,
        low: trade_price_bucket_update.low,
        close: trade_price_bucket_update.close,
    }
}

fn handle_aggregated_orderbook_update(aggregated_orderbook_update: AggregatedOrderBookUpdate) {
    for ask in aggregated_orderbook_update.asks {
        println!(
            "Ask Price: {}, quantity: {}, side: {}",
            ask.price, ask.quantity, ask.side
        );
    }

    for bid in aggregated_orderbook_update.bids {
        println!(
            "Bid Price: {}, quantity: {}, side: {}",
            bid.price, bid.quantity, bid.side
        );
    }
}

fn handle_balance_update(balance_update: BalanceUpdate) {
    println!(
        "{}: {}",
        balance_update.currency.symbol.bright_green(),
        balance_update.available.bright_blue()
    );
}

fn handle_orderbook_snapshot(orderbook_data: OrderBookData) {
    // Access parsed fields from the struct
    println!("Last Change: {}", orderbook_data.last_change);
    println!("Sequence Number: {}", orderbook_data.sn);
    println!("Checksum: {}", orderbook_data.checksum);
    // Iterate over Asks and Bids
    for ask in orderbook_data.asks {
        println!("Ask Price: {}", ask.price);
        for order in ask.orders {
            println!(
                "  Order ID: {}, Quantity: {}",
                order.order_id, order.quantity
            );
        }
    }

    for bid in orderbook_data.bids {
        println!("Bid Price: {}", bid.price);
        for order in bid.orders {
            println!(
                "  Order ID: {}, Quantity: {}",
                order.order_id, order.quantity
            );
        }
    }
}

async fn get_historical_sixty_second_mark_price_buckets_for_pair(
    currency_pair: &String,
    start_time: String,
    end_time: String,
) -> Result<(), reqwest::Error> {
    let request_url = format!("https://api.valr.com/v1/public/{}/markprice/buckets?startTime={}&endTime={}&periodSeconds={}", currency_pair, start_time, end_time, FIVE_MINUTE_BUCKET_SECONDS);
    let client = reqwest::Client::new();
    let response = client.get(request_url).send().await?;
    let mark_price_buckets: Vec<MarkPriceBucket> = response.json().await?;
    let mut bpw = BUCKET_PRICES.write().await;
    for mark_price_bucket in mark_price_buckets {
        bpw.push(mark_price_bucket)
    }
    drop(bpw);
    tokio::spawn(async move {
        let bpr = BUCKET_PRICES.read().await;
        println!("{:?}", bpr);
        drop(bpr);
    })
    .await
    .expect("The spawned task to read Bucket prices has panicked");
    Ok(())
}

//Not necessary because the first subscription always returns all the open orders
//but leaving as an example
async fn get_open_orders_for_pair(
    api_key: &str,
    api_secret: &str,
    currency_pair: &String,
) -> Result<(), reqwest::Error> {
    let request_url = String::from("https://api.valr.com/v1/orders/open");
    let response = create_http_request(
        request_url,
        api_key,
        api_secret,
        "/v1/orders/open",
        "GET",
        None,
    )
    .send()
    .await;

    let orders: Vec<Order> = response.unwrap().json().await?;
    let mut orders_writer = ORDERS.write().await;
    orders
        .iter()
        .filter(|o| o.currency_pair.eq(currency_pair))
        .for_each(|o| {
            println!("{:?}", o);
            orders_writer.push(o.clone());
        });
    // for order in orders {
    //     println!("{:?}", order);
    //     orders_writer.push(order);
    // }
    drop(orders_writer);
    Ok(())
}