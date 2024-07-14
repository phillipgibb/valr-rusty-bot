#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[cfg(test)]
mod tests {
    use futures_util::TryFutureExt;
    use log::error;
    use serde_json::{json, Value};
    use crate::config::{ConfigProvider, DotEnvConfigProvider};
    use crate::rusty_bot_models::SubAccountResponse;
    use crate::strategies::break_of_structure::helper::create_http_request;

    #[tokio::test]
    async fn create_sub_account() -> Result<(), reqwest::Error> {
        let env_config_provider = DotEnvConfigProvider::new();
        let config = env_config_provider.get_config();
        let request_url = String::from("https://api.valr.com/v1/account/subaccount");

        let msg = json!({
            "label": "Test5"
        });

        let response = create_http_request(
            request_url,
            &config.api_key,
            &config.api_secret,
            "/v1/account/subaccount",
            "POST",
            Option::from(msg.to_string()),
        ).send().await;

        match response.unwrap().error_for_status() {
            Ok(_response) => {
                // Handle successful response
                let sub_account_response = _response.json::<SubAccountResponse>().await.expect("TODO: panic message");
                println!("{:?}", sub_account_response);
                delete_sub_account(sub_account_response.id.parse::<i64>().unwrap()).await.expect("TODO: panic message");
                Ok(())
            }
            Err(error) => {
                if error.is_timeout() {
                    // Handle timeout error
                    Ok(println!("Request timed out"))
                } else if error.is_connect() {
                    // Handle connection error
                    Ok(println!("Network connection error"))
                } else {
                    // Handle other errors
                    println!("Error: {:?}", error.status());
                    Ok(())
                }
            }
        }
    }

    async fn delete_sub_account(id: i64) -> Result<(), reqwest::Error> {
        let env_config_provider = DotEnvConfigProvider::new();
        let config = env_config_provider.get_config();
        let request_url = String::from("https://api.valr.com/v1/account/subaccount");
        let msg = json!({
            "subAccountPublicId": id
        });

        let response = create_http_request(
            request_url,
            &config.api_key,
            &config.api_secret,
            "/v1/account/subaccount",
            "DELETE",
            Option::from(msg.to_string()),
        )
            .send()
            .await;

        match response.unwrap().error_for_status() {
            Ok(_response) => {
                // Handle successful response
                println!("DELETED Successfully");
                Ok(())
            }
            Err(error) => {
                if error.is_timeout() {
                    // Handle timeout error
                    Ok(println!("Request timed out"))
                } else if error.is_connect() {
                    // Handle connection error
                    Ok(println!("Network connection error"))
                } else {
                    // Handle other errors
                    println!("Error: {:?}", error.status());
                    Ok(())
                }
            }
        }
    }
}
