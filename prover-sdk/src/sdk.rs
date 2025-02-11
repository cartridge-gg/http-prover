use crate::{access_key::ProverAccessKey, errors::SdkErrors};
use chrono::Utc;
use common::{
    prover_input::{Cairo0ProverInput, CairoProverInput, LayoutBridgeInput, ProverInput},
    requests::AddKeyRequest,
    snos_input::SnosPieInput,
    HttpProverData,
};
use ed25519_dalek::{ed25519::signature::Signer, VerifyingKey};

use futures::StreamExt;
use rand::Rng;
use reqwest::{Client, Response};
use serde::Deserialize;
use url::Url;
#[derive(Debug, Clone)]
/// ProverSDK is a struct representing a client for interacting with the Prover service.
pub struct ProverSDK {
    pub client: Client,
    pub prover_cairo0: Url,
    pub prover_cairo: Url,
    pub run_cairo0: Url,
    pub run_cairo: Url,
    pub layout_bridge: Url,
    pub snos_pie_gen: Url,
    pub verify: Url,
    pub get_job: Url,
    pub register: Url,
    pub sse: Url,
    pub authority: ProverAccessKey,
}

#[derive(Deserialize)]
pub struct JobId {
    pub job_id: u64,
}

impl ProverSDK {
    pub async fn new(url: Url, access_key: ProverAccessKey) -> Result<Self, SdkErrors> {
        let url = if !url.as_str().ends_with('/') {
            let mut url_with_slash = url.clone();
            url_with_slash.set_path(&format!("{}/", url.path()));
            url_with_slash
        } else {
            url
        };
        let client = reqwest::Client::new();

        Ok(ProverSDK {
            client,
            prover_cairo0: url.join("prove/cairo0")?,
            prover_cairo: url.join("prove/cairo")?,
            run_cairo0: url.join("run/cairo0")?,
            run_cairo: url.join("run/cairo")?,
            layout_bridge: url.join("layout-bridge")?,
            snos_pie_gen: url.join("run/snos")?,
            verify: url.join("verify")?,
            get_job: url.join("get-job")?,
            register: url.join("register")?,
            sse: url.join("sse")?,
            authority: access_key,
        })
    }

    async fn send_prover_request<T: HttpProverData>(
        &self,
        data: T,
        url: &Url,
    ) -> Result<u64, SdkErrors> {
        let nonce = rand::thread_rng().gen::<u64>();
        let current_time = Utc::now().to_rfc3339();
        let signature = data.sign(self.authority.0.clone(), current_time.clone(), nonce);

        let response = self
            .client
            .post(url.clone())
            .header("X-Signature", signature)
            .header("X-Timestamp", current_time)
            .header("X-Nonce", nonce)
            .json(&data.to_json_value())
            .send()
            .await?;

        if !response.status().is_success() {
            let response_data: String = response.text().await?;
            tracing::error!("{}", response_data);
            return Err(SdkErrors::ProveResponseError(response_data));
        }

        let response_data = response.text().await?;
        let job = serde_json::from_str::<JobId>(&response_data)?;
        Ok(job.job_id)
    }

    pub async fn prove_cairo0(&self, data: Cairo0ProverInput) -> Result<u64, SdkErrors> {
        if !data.layout.is_bootloadable()
            && matches!(data.run_mode, common::prover_input::RunMode::Bootload)
        {
            return Err(SdkErrors::BootloaderError);
        }
        self.prove(ProverInput::Cairo0(data), self.prover_cairo0.clone())
            .await
    }

    pub async fn prove_cairo(&self, data: CairoProverInput) -> Result<u64, SdkErrors> {
        if !data.layout.is_bootloadable()
            && matches!(data.run_mode, common::prover_input::RunMode::Bootload)
        {
            return Err(SdkErrors::BootloaderError);
        }
        self.prove(ProverInput::Cairo(data), self.prover_cairo.clone())
            .await
    }

    async fn prove(&self, data: ProverInput, url: Url) -> Result<u64, SdkErrors> {
        self.send_prover_request(data, &url).await
    }

    pub async fn run_cairo0(&self, data: Cairo0ProverInput) -> Result<u64, SdkErrors> {
        if !data.layout.is_bootloadable()
            && matches!(data.run_mode, common::prover_input::RunMode::Bootload)
        {
            return Err(SdkErrors::BootloaderError);
        }
        self.run(ProverInput::Cairo0(data), self.run_cairo0.clone())
            .await
    }

    pub async fn run_cairo(&self, data: CairoProverInput) -> Result<u64, SdkErrors> {
        if !data.layout.is_bootloadable()
            && matches!(data.run_mode, common::prover_input::RunMode::Bootload)
        {
            return Err(SdkErrors::BootloaderError);
        }
        self.run(ProverInput::Cairo(data), self.run_cairo.clone())
            .await
    }

    async fn run(&self, data: ProverInput, url: Url) -> Result<u64, SdkErrors> {
        self.send_prover_request(data, &url).await
    }

    pub async fn snos_pie_gen(&self, data: SnosPieInput) -> Result<u64, SdkErrors> {
        self.send_prover_request(data, &self.snos_pie_gen).await
    }
    pub async fn layout_bridge(&self, data: LayoutBridgeInput) -> Result<u64, SdkErrors> {
        self.send_prover_request(data, &self.layout_bridge).await
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

    pub async fn sse(&self, job_id: u64) -> Result<(), SdkErrors> {
        let url = format!("{}?job_id={}", self.sse.clone().as_str(), job_id);
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(SdkErrors::SSEError(format!(
                "Failed to get SSE with status code: {}",
                response.status(),
            )));
        }

        let mut stream = response.bytes_stream();
        while let Some(_item) = stream.next().await {}
        Ok(())
    }
}
