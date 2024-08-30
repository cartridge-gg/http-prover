use crate::{access_key::ProverAccessKey, errors::SdkErrors, sdk::ProverSDK};
use common::{
    models::JWTResponse,
    requests::{GenerateNonceRequest, ValidateSignatureRequest},
};
use ed25519_dalek::{Signature, Signer, VerifyingKey};
use reqwest::{cookie::Jar, Client};
use serde_json::Value;
use std::sync::Arc;
use url::Url;

#[derive(Debug)]
pub struct ProverSDKBuilder {
    client: Client,
    base_url: Url,
    auth: Url,
    signing_key: Option<ProverAccessKey>,
    jwt_token: Option<String>,
}
impl ProverSDKBuilder {
    pub fn new(auth: Url, base_url: Url) -> Self {
        ProverSDKBuilder {
            client: Client::new(),
            auth,
            base_url,
            signing_key: None,
            jwt_token: None,
        }
    }
    pub async fn get_nonce(&self, public_key: &VerifyingKey) -> Result<String, SdkErrors> {
        let nonce_req = GenerateNonceRequest {
            public_key: serde_json::to_string(&public_key)?,
        };
        let response = self.client.get(self.auth.clone()).query(&nonce_req).send().await?;

        if !response.status().is_success() {
            return Err(SdkErrors::NonceRequestFailed(format!(
                "Failed to get nonce from URL: {} with status code: {}",
                self.auth,
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
    pub async fn validate_signature(
        &self,
        signed_nonce: Signature,
        nonce: String,
    ) -> Result<JWTResponse, SdkErrors> {
        let request = ValidateSignatureRequest {
            signature: signed_nonce,
            nonce,
        };
        let response = self
            .client
            .post(self.auth.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(SdkErrors::ValidateSignatureResponseError(format!(
                "Failed to validate signature with status code: {}",
                response.status(),
            )));
        }
        let json_body: Value = response.json().await?;
        let jwt_token = json_body["jwt_token"]
            .as_str()
            .ok_or(SdkErrors::JWTTokenNotFound)?
            .to_string();
        let expiration = json_body["expiration"]
            .as_u64()
            .ok_or(SdkErrors::JWTExpirationNotFound)?;
        Ok(JWTResponse {
            jwt_token: jwt_token,
            expiration: expiration,
        })
    }
    
    pub async fn auth(mut self, signing_key: ProverAccessKey) -> Result<Self, SdkErrors> {
        self.signing_key = Some(signing_key);
        let jwt_response = self.get_jwt_token().await?;
        self.jwt_token = Some(jwt_response.jwt_token);
        Ok(self)
    }

    async fn get_jwt_token(&self) -> Result<JWTResponse, SdkErrors> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or(SdkErrors::SigningKeyNotFound)?;

        let public_key = signing_key.0.verifying_key();

        let nonce = self.get_nonce(&public_key).await?;

        let signed_nonce = signing_key.0.sign(nonce.as_bytes());

        self.validate_signature(signed_nonce, nonce).await
    }

    pub fn build(self) -> Result<ProverSDK, SdkErrors> {
        let signing_key = self.signing_key.ok_or(SdkErrors::SigningKeyNotFound)?;

        let jwt_token = self.jwt_token.ok_or(SdkErrors::JWTTokenNotFound)?;

        let prover = self.base_url.join("")?;

        let cookie_jar = Arc::new(Jar::default());
        let secure_attribute = if prover.scheme() == "https" {
            "Secure; "
        } else {
            ""
        };

        cookie_jar.add_cookie_str(
            &format!(
                "jwt_token={}; HttpOnly; {}  SameSite=None; Path=/",
                jwt_token, secure_attribute
            ),
            &prover,
        );
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_jar)
            .build()?;

        Ok(ProverSDK {
            client,
            prover_cairo0: self.base_url.join("/prove/cairo0")?,
            prover_cairo: self.base_url.join("/prove/cairo")?,
            verify: self.base_url.join("/verify")?,
            get_job: self.base_url.join("/get-job")?,
            authority: signing_key,
        })
    }
}
