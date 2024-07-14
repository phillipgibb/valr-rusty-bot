use std::ops::Deref;

pub trait ConfigProvider {
    fn get_config(&self) -> &Config;
}
pub struct Config {
    pub api_key: String,
    pub api_secret: String,
    pub market: String,
    pub strategy: String,
}

pub struct DotEnvConfigProvider(Config);

impl DotEnvConfigProvider {
    pub fn new() -> Self {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
        let api_key = env::var("API_KEY").expect("Missing API_KEY");
        let api_secret = env::var("API_SECRET").expect("Missing API_SECRET");
        let market = env::var("MARKET").expect("Missing MARKET");
        let strategy = env::var("STRATEGY").expect("Missing STRATEGY");
        
        let config = Config {
             api_key,
             api_secret,
             market,
             strategy
        };

        DotEnvConfigProvider(config)
    }
}

impl ConfigProvider for DotEnvConfigProvider {
    fn get_config(&self) -> &Config {
        &self.0
    }
}

impl Default for DotEnvConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}