// #[path = "./strategies/break_of_structure.rs"]
// mod break_of_structure;
#[path = "rusty_bot_models.rs"]
mod rusty_bot_models;
use std::env;
use std::sync::Arc;
use std::time::SystemTime;

use dotenv::dotenv;
use hmac::{Hmac, KeyInit, Mac};
use http::Uri;
use reqwest::RequestBuilder;
use serde::Deserialize;
use sha2::Sha512;
use tokio::sync::{RwLock};
use tungstenite::client::IntoClientRequest;
use crate::helper::rusty_bot_models::MarkPriceBucket;
// use crate::rusty_bot_models::MarkPriceBucket;

#[derive(Deserialize)]
pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub market: String,
    pub strategy: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let api_key = env::var("API_KEY").unwrap();
        let api_secret = env::var("API_SECRET").unwrap();
        let market = env::var("MARKET").unwrap();
        let strategy  = env::var("STRATEGY").unwrap();
        Ok(Config {
            api_key,
            api_secret,
            market,
            strategy
        })
    }
}

pub async fn execute_strategy(strategy: String, bucket_prices:  Vec<MarkPriceBucket>, asks: &Arc<RwLock<Vec<Vec<String>>>>, bids: &Arc<RwLock<Vec<Vec<String>>>>) {
    match strategy.as_str() {
        "break_of_structure" => {
            // break_of_structure::test_for_break_of_structure(bucket_prices, &asks, &bids).await;
        },
        _ => {
            println!("Strategy {} not supported", strategy)
        }
    }
}

pub fn load_env() -> Config {
    println!("Loading .env variables!!");
    dotenv().ok();
    envy::from_env::<Config>()
        .expect("Please provide API env vars")
}

pub fn strip_slashes(s: &str) -> Option<String> {
    let mut n = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        n.push(match c {
            '\\' => chars.next()?,
            c => c,
        });
    }
    Some(n)
}

pub fn create_ws_request(url: Uri, api_key: &String, api_secret: &String, path: String, verb: String, body: Option<String>) -> tungstenite::handshake::client::Request {
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let sig = api_sign(api_secret.as_bytes(), timestamp.to_string(), &verb, path, Option::from(body));
    let mut request = IntoClientRequest::into_client_request(url).unwrap();
    let headers = request.headers_mut();
    headers.insert("X-VALR-API-KEY", api_key.parse().unwrap());
    headers.insert("X-VALR-SIGNATURE", sig.parse().unwrap());
    headers.insert("X-VALR-TIMESTAMP", format!("{}", &timestamp).parse().unwrap());
    headers.insert("X-VALR-API-KEY", api_key.parse().unwrap());
    headers.insert("X-VALR-SIGNATURE", sig.parse().unwrap());
    headers.insert("X-VALR-TIMESTAMP", format!("{}", &timestamp).parse().unwrap());
    request
}

pub fn create_http_request(url: String, api_key: &String, api_secret: &String, path: String, verb: &String, body: Option<String>) -> RequestBuilder {
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let sig = api_sign(api_secret.as_bytes(), timestamp.to_string(), verb, path, body.clone());
    let client = reqwest::Client::new();
    let request_builder = match verb.clone().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url).body(body.unwrap().clone()),
        "PUT" => client.put(url).body(body.unwrap().clone()),
        "DELETE" => client.delete(url).body(body.unwrap().clone()),
        _ => panic!("Verb: {} not supported", verb)
    };

    request_builder.header("X-VALR-API-KEY", api_key.clone())
    .header("X-VALR-SIGNATURE", sig.clone())
    .header("X-VALR-TIMESTAMP", format!("{}", &timestamp).clone())
    .header("X-VALR-API-KEY", api_key.clone())
    .header("X-VALR-SIGNATURE", sig.clone())
    .header("X-VALR-TIMESTAMP", format!("{}", timestamp.clone()))
}
//
// pub fn create_http_post(url: String, api_key: &String, api_secret: &String, path: String, body: String) -> RequestBuilder {
//     let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
//     let sig = api_sign(api_secret.as_bytes(), timestamp.to_string(), String::from("POST"), path, Option::from(body.clone()));
//     let client = reqwest::Client::new();
//     client.post(url).body(body.clone())
//         .header("X-VALR-API-KEY", api_key.clone())
//         .header("X-VALR-SIGNATURE", sig.clone())
//         .header("X-VALR-TIMESTAMP", format!("{}", &timestamp).clone())
//         .header("X-VALR-API-KEY", api_key.clone())
//         .header("X-VALR-SIGNATURE", sig.clone())
//         .header("X-VALR-TIMESTAMP", format!("{}", timestamp.clone()))
//         .header("Content-length", body.clone().len())
//         .header("Content-Type", "json")
// }
//
// pub fn create_http_delete(url: String, api_key: &String, api_secret: &String, path: String, body: String) -> RequestBuilder {
//     let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
//     let sig = api_sign(api_secret.as_bytes(), timestamp.to_string(), String::from("DELETE"), path, Option::from(body.clone()));
//     let client = reqwest::Client::new();
//     client.delete(url).body(body.clone())
//         .header("X-VALR-API-KEY", api_key.clone())
//         .header("X-VALR-SIGNATURE", sig.clone())
//         .header("X-VALR-TIMESTAMP", format!("{}", &timestamp).clone())
//         .header("X-VALR-API-KEY", api_key.clone())
//         .header("X-VALR-SIGNATURE", sig.clone())
//         .header("X-VALR-TIMESTAMP", format!("{}", timestamp.clone()))
//         .header("Content-length", body.clone().len())
//         .header("Content-Type", "json")
// }

pub fn api_sign(secret: &[u8], timestamp: String, verb: &String, path: String, data: Option<String>) -> String {
    let mut mac = Hmac::<Sha512>::new_from_slice(secret).unwrap();
    mac.update(&timestamp.into_bytes());
    mac.update(&*verb.clone().into_bytes());
    mac.update(&path.into_bytes());
    if data.is_some() { mac.update(&data.unwrap().into_bytes()) }
    let result = mac.finalize();
    format!("{}", hex::encode(result.into_bytes()))
}

