use crate::models::{bytes_to_hex_string, JWTResponse};
use crate::prover_sdk::ProverSDK;
use crate::errors::ProverSdkErrors;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use reqwest::{cookie::Jar, Client, Url};
use serde_json::{json, Value};
use std::sync::Arc;

/// ProverSDKBuilder is a builder for constructing a ProverSDK instance.
#[derive(Debug)]
pub struct ProverSDKBuilder {
    client: Client,
    url_auth: String,
    url_prover: String,
    signing_key: Option<SigningKey>,
    jwt_token: Option<String>,
}

impl ProverSDKBuilder {
    /// Creates a new ProverSDKBuilder instance.
    ///
    /// # Arguments
    ///
    /// * `url_auth` - The URL of the authentication service.
    /// * `url_prover` - The URL of the Prover service.
    ///
    /// # Returns
    ///
    /// Returns a new instance of ProverSDKBuilder.
    pub fn new(url_auth: &str, url_prover: &str) -> Self {
        ProverSDKBuilder {
            client: Client::new(),
            url_auth: url_auth.to_string(),
            url_prover: url_prover.to_string(),
            signing_key: None,
            jwt_token: None,
        }
    }

    /// Authenticates with the authentication service using the provided private key.
    ///
    /// # Arguments
    ///
    /// * `private_key_hex` - The hexadecimal representation of the private key.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProverSDKBuilder instance with authentication
    /// information if successful, or a ProverSdkErrors if an error occurs.
    pub async fn auth(mut self, private_key_hex: &str) -> Result<Self, ProverSdkErrors> {
        // Convert the hexadecimal private key string into bytes
        let private_key_bytes = hex::decode(private_key_hex)?;
        let mut private_key_array = [0u8; 32];
        private_key_array.copy_from_slice(&private_key_bytes);
        let signing_key = SigningKey::from_bytes(&private_key_array);
        self.signing_key = Some(signing_key);
        let jwt_response = self.get_jwt_token().await?;
        self.jwt_token = Some(jwt_response.jwt_token);
        Ok(self)
    }

    /// Asynchronously retrieves a JWT token from the authentication service using the provided signing key.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a JWTResponse if successful, or a ProverSdkErrors if an error occurs.
    async fn get_jwt_token(&self) -> Result<JWTResponse, ProverSdkErrors> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or(ProverSdkErrors::SigningKeyNotFound)?;
        let public_key = signing_key.verifying_key();

        let nonce = self.get_nonce(&public_key).await?;

        let signed_nonce = signing_key.sign(nonce.as_bytes());

        self.validate_signature(&public_key, &nonce, &signed_nonce)
            .await
    }

    /// Asynchronously retrieves a nonce from the authentication service using the provided public key.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key used to request the nonce. dalek_ed25519 VerifyingKey
    ///
    /// # Returns
    ///
    /// Returns a Result containing a nonce string if successful, or a ProverSdkErrors if an error occurs.
    async fn get_nonce(&self, public_key: &VerifyingKey) -> Result<String, ProverSdkErrors> {
        let url_with_params = format!(
            "{}?public_key={}",
            &self.url_auth,
            bytes_to_hex_string(public_key.as_bytes())
        );

        let response = match self.client.get(&url_with_params).send().await {
            Ok(response) => response,
            Err(reqwest_error) => {
                return Err(ProverSdkErrors::NonceRequestFailed(format!(
                    "Failed to send HTTP request to URL: {}. Error: {}",
                    url_with_params, reqwest_error
                )));
            }
        };
        if !response.status().is_success() {
            // If the status is not successful, return an appropriate error
            return Err(ProverSdkErrors::NonceRequestFailed(format!(
                "Failed to get nonce from URL: {} with status code: {}",
                url_with_params,
                response.status()
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            ProverSdkErrors::NonceRequestFailed(format!(
                "Failed to read response text from URL: {}. Error: {}",
                url_with_params, e
            ))
        })?;   

        let json_body: Value = serde_json::from_str(&response_text).map_err(|e| {
            ProverSdkErrors::JsonParsingFailed(format!(
                "Failed to parse JSON response from URL: {}. Error: {}",
                url_with_params, e
            ))
        })?;

        let nonce = json_body["nonce"]
            .as_str()
            .ok_or(ProverSdkErrors::NonceNotFound)?
            .to_string();

        Ok(nonce)
    }

    /// Asynchronously validates the signature of the provided nonce and retrieves a JWT token from the authentication service.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key used to request the nonce.
    /// * `nonce` - The nonce received from the authentication service.
    /// * `signed_nonce` - The signature of the nonce.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a JWTResponse if successful, or a ProverSdkErrors if an error occurs.
    async fn validate_signature(
        &self,
        public_key: &VerifyingKey,
        nonce: &String,
        signed_nonce: &Signature,
    ) -> Result<JWTResponse, ProverSdkErrors> {
        let data = json!({
            "public_key": bytes_to_hex_string(&public_key.to_bytes()),
            "nonce": nonce,
            "signature": bytes_to_hex_string(&signed_nonce.to_bytes()),
        });

        let response = match self.client
            .post(&self.url_auth)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await 
            {
            Ok(response) => response,
            Err(reqwest_error) => {
                return Err(ProverSdkErrors::ValidateSignatureRequestFailed(format!(
                    "Failed to send HTTP request to URL: {}. Error: {}",
                    &self.url_auth, reqwest_error
                )));
            }
        };
        
        if !response.status().is_success() {
            return Err(ProverSdkErrors::ValidateSignatureResponseError(format!(
                "Received unsuccessful status code ({}) from URL: {}",
                response.status(), &self.url_auth
            )));
        }

        let json_body: Value = match response.json().await {
            Ok(json_body) => json_body,
            Err(json_error) => {
                return Err(ProverSdkErrors::JsonParsingFailed(format!(
                    "Failed to parse JSON response from URL: {}. Error: {}",
                    &self.url_auth, json_error
                )));
            }
        };

        let jwt_token = json_body["jwt_token"]
            .as_str()
            .ok_or(ProverSdkErrors::JwtTokenNotFound)?
            .to_string();
        let expiration = json_body["expiration"]
            .as_u64()
            .ok_or(ProverSdkErrors::ExpirationNotFound)?;

        Ok(JWTResponse {
            jwt_token,
            expiration,
        })
    }

    /// Builds the ProverSDK instance.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the constructed ProverSDK instance if successful,
    /// or a ProverSdkErrors if an error occurs.
    pub fn build(self) -> Result<ProverSDK, ProverSdkErrors> {
        let _signing_key = self
            .signing_key
            .ok_or(ProverSdkErrors::SigningKeyNotFound)?;
        let jwt_token = self.jwt_token.ok_or(ProverSdkErrors::JwtTokenNotFound)?;

        let url_prover = Url::parse(&self.url_prover)?;

        let jar = Jar::default();
        jar.add_cookie_str(
            &format!("jwt_token={}; HttpOnly; Secure; Path=/", jwt_token),
            &url_prover,
        );

        let client = reqwest::Client::builder()
            .cookie_provider(Arc::new(jar))
            .build().map_err(|e| ProverSdkErrors::ReqwestBuildError(format!("Failed to build reqwest client: {}", e)))?;


        Ok(ProverSDK {
            client,
            url_prover: self.url_prover,
        })
    }
}