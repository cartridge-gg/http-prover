use ed25519_dalek::VerifyingKey;
use reqwest::Client;
use serde_json::Value;
use url::Url;

use crate::{access_key::ProverAccessKey, errors::SdkErrors};

#[derive(Debug)]
pub struct ProverSDKBuilder {
    client: Client,
    auth: Url,
    signing_key: Option<ProverAccessKey>,
    jwt_token: Option<String>,
}
impl ProverSDKBuilder {
    pub fn new(auth: Url) -> Self {
        ProverSDKBuilder {
            client: Client::new(),
            auth,
            signing_key: None,
            jwt_token: None,
        }
    }
    pub async fn get_nonce(&self, public_key: &VerifyingKey) -> Result<String, SdkErrors> {
        let url_with_params = format!(
            "{}?public_key={}",
            self.auth,
            prefix_hex::encode(public_key.as_bytes())
        );
        println!("url_with_params: {}", url_with_params);
        let response = self.client.get(&url_with_params).send().await?;

        if !response.status().is_success() {
            return Err(SdkErrors::NonceRequestFailed(format!(
                "Failed to get nonce from URL: {} with status code: {}",
                url_with_params,
                response.status(),
            )));
        }

        let response_text = response.text().await?;

        let json_body: Value = serde_json::from_str(&response_text)?;

        let nonce = json_body["nonce"]
            .as_str()
            .ok_or(SdkErrors::NonceNotFound)?
            .to_string();

        Ok(nonce)
    }
}
