pub trait ConfigProvider {
    fn get_config(&self) -> &Config;
}
#[derive(Debug, Default)]
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
        let config = Config {
             api_key: env::var("API_KEY").expect("Missing API_KEY"),
             api_secret : env::var("API_SECRET").expect("Missing API_SECRET"),
             market: env::var("MARKET").expect("Missing MARKET"),
             strategy: env::var("STRATEGY").expect("Missing STRATEGY")
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