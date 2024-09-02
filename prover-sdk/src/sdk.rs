use common::{requests::AddKeyRequest, ProverInput};
use ed25519_dalek::{ed25519::signature::SignerMut, VerifyingKey};
use reqwest::{Client, Response};
use url::Url;

use crate::{access_key::ProverAccessKey, errors::SdkErrors, sdk_builder::ProverSDKBuilder};
#[derive(Debug, Clone)]
/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub prover_cairo0: Url,
    pub prover_cairo: Url,
    pub verify: Url,
    pub get_job: Url,
    pub register: Url,
    pub authority: ProverAccessKey,
}

impl ProverSDK {
    pub async fn new(url: Url, access_key: ProverAccessKey) -> Result<Self, SdkErrors> {
        let auth_url = url.join("auth")?;
        ProverSDKBuilder::new(auth_url, url)
            .auth(access_key)
            .await?
            .build()
    }

    pub async fn prove_cairo0<T>(&self, data: T) -> Result<String, SdkErrors>
    where
        T: ProverInput + Send + 'static,
    {
        self.prove(data, self.prover_cairo0.clone()).await
    }

    pub async fn prove_cairo<T>(&self, data: T) -> Result<String, SdkErrors>
    where
        T: ProverInput + Send + 'static,
    {
        self.prove(data, self.prover_cairo.clone()).await
    }

    async fn prove<T>(&self, data: T, url: Url) -> Result<String, SdkErrors>
    where
        T: ProverInput + Send + 'static,
    {
        let response = self
            .client
            .post(url.clone())
            .json(&data.serialize())
            .send()
            .await?;

        if !response.status().is_success() {
            let response_data: String = response.text().await?;
            tracing::error!("{}", response_data);
            return Err(SdkErrors::ProveResponseError(response_data));
        }
        let response_data = response.text().await?;

        Ok(response_data)
    }
    pub async fn verify(self, proof: String) -> Result<String, SdkErrors> {
        let response = self
            .client
            .post(self.verify.clone())
            .json(&proof)
            .send()
            .await?;
        let response_data = response.text().await?;
        Ok(response_data)
    }
    pub async fn get_job(&self, job_id: u64) -> Result<Response, SdkErrors> {
        let url = format!("{}/{}", self.get_job.clone().as_str(), job_id);
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let response_data: String = response.text().await?;
            tracing::error!("{}", response_data);
            return Err(SdkErrors::GetJobResponseError(response_data));
        }
        Ok(response)
    }
    pub async fn register(&mut self, key: VerifyingKey) -> Result<(), SdkErrors> {
        let signature = self.authority.0.sign(key.as_bytes());
        let request = AddKeyRequest {
            signature,
            new_key: key,
            authority: self.authority.0.verifying_key(),
        };
        let response = self
            .client
            .post(self.register.clone())
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(SdkErrors::RegisterResponseError(format!(
                "Failed to register key with status code: {}",
                response.status(),
            )));
        }
        Ok(())
    }
}
