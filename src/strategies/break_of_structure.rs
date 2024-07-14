// #[path = "../rusty_bot_models.rs"]
// pub mod rusty_bot_models;

use std::sync::Arc;
use log::warn;

use tokio::sync::{RwLock};
use crate::rusty_bot_models::MarkPriceBucket;

#[path = "../helper.rs"]
pub mod helper;

pub async fn test_for_break_of_structure(bucket_prices: Vec<MarkPriceBucket>, asks: &Arc<RwLock<Vec<Vec<String>>>>, bids: &Arc<RwLock<Vec<Vec<String>>>>) {
    let width = 3; //width of the spread under consideration
    let length = (width * 2) + 1;
    if bucket_prices.is_empty() {
        warn!("No Buckets available");
        return;
    }

    let asks_reader = asks.read().await;
    let bids_reader = bids.read().await;

    if asks_reader.is_empty()  || bids_reader.is_empty(){
        warn!("No Asks or Bids available");
        return;
    }
    let best_ask = asks_reader.first().unwrap();
    let best_bid = bids_reader.first().unwrap();

    let current_index = bucket_prices.len() - width - 1;
    let previous_close = bucket_prices.last().unwrap().close;

    for i in 1..width {
        let left_neighbor_index = current_index - i;
        let right_neighbor_index = current_index + i;

        let current = bucket_prices.get(i).unwrap();
        let left = bucket_prices.get(left_neighbor_index).unwrap();
        let right = bucket_prices.get(right_neighbor_index).unwrap();

        let current_high = current.high;
        let current_low = current.low;
        let current_close = current.close;

        let right_high = right.high;
        let right_low = right.low;
        let right_close = right.close;

        let left_high = left.high;
        let left_low = left.low;
        let left_close = left.close;

        let is_high_swing = !(current_high <= left_high || right_high < current_high);

        let is_low_swing = !(current_low >= left_low || right_low > current_low);

        let swing_high: f64 = if is_high_swing {
            current_high
        } else {
            -1.0f64
        };
        let swing_low: f64 = if is_low_swing {
            current_low
        } else {
            -1.0
        };

        
        let best_bid_price = &best_bid[0].parse::<f64>().unwrap();
        let best_ask_price = &best_ask[0].parse::<f64>().unwrap();

        if is_high_swing {
            println!("IS HIGH SWING");
            println!();
            println!(
                "LEFT    BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                left_close, left_high, left_low
            );
            println!(
                "CURRENT BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                current_close, current_high, current_low
            );
            println!(
                "RIGHT   BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                right_close, right_high, right_low
            );
            println!();
            println!("best_bid_price: {}", best_bid_price);
            println!("best_ask_price: {}", best_ask_price);
            println!();
            println!("swing high: {} > 0 = {}", swing_high, swing_high > 0f64);
            println!(
                "best_bid_price: {} > swing_high: {} = {}",
                best_bid_price,
                swing_high,
                *best_bid_price > swing_high
            );
            println!(
                "previous_close > swing_high? ({} > {}): {}",
                previous_close,
                swing_high,
                previous_close > swing_high
            );
            println!();
        }

        if is_low_swing {
            println!("IS LOW SWING");
            println!();
            println!(
                "LEFT    BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                left_close, left_high, left_low
            );
            println!(
                "CURRENT BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                current_close, current_high, current_low
            );
            println!(
                "RIGHT   BUCKET CLOSE: {}, HIGH: {}, LOW: {}",
                right_close, right_high, right_low
            );
            println!();
            println!("best_bid_price: {}", best_bid_price);
            println!("best_ask_price: {}", best_ask_price);
            println!();
            println!("swing low: {} > 0 = {}", swing_low, swing_low > 0f64);
            println!(
                "best_ask_price: {} < swing_low: {} = {}",
                best_ask_price,
                swing_low,
                *best_ask_price > swing_low
            );
            println!(
                "previous_close < swing_low? ({} < {}) : {}",
                previous_close,
                swing_low,
                previous_close < swing_low
            );
            println!();
        }

        if swing_high > 0f64 && best_bid_price > &swing_high && previous_close > swing_high {
            buy(*best_ask_price, best_ask[1].clone()).await;
        } else if swing_low > 0f64 && *best_ask_price < swing_low && previous_close < swing_low {
            sell(*best_bid_price, best_bid[1].clone()).await;
        }
    }
    drop(bids_reader);
    drop(asks_reader);
}

async fn sell(best_bid_price: f64, quantity: String) {
    println!(
        "Place SELL at price: {} and quantity: {}",
        best_bid_price, quantity
    );
    //drop sells?
}

async fn buy(best_ask_price: f64, quantity: String) {
    println!(
        "Place SELL at price: {} and quantity: {}",
        best_ask_price, quantity
    );
    //drop buys?
}
