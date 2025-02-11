#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::HttpProverData;
    use prover_sdk::errors::SdkErrors::ProveResponseError;
    use prover_sdk::{
        access_key::ProverAccessKey, errors::SdkErrors, sdk::ProverSDK, CairoCompiledProgram,
        CairoProverInput, Layout, ProverInput, RunMode,
    };
    use reqwest::Client;
    use starknet_types_core::felt::Felt;
    use url::Url;
    #[tokio::test]
    async fn test_authorized_access() {
        let private_key = std::env::var("PRIVATE_KEY").unwrap();
        let url = std::env::var("PROVER_URL").unwrap();
        let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
        let url = Url::parse(&url).unwrap();
        let sdk = ProverSDK::new(url, access_key).await.unwrap();
        let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
        let program: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
        let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
        let mut program_input: Vec<Felt> = Vec::new();
        for part in program_input_string.split(',') {
            let felt = Felt::from_dec_str(part).unwrap();
            program_input.push(felt);
        }
        let data = CairoProverInput {
            program,
            layout: Layout::Recursive,
            program_input,
            n_queries: Some(16),
            pow_bits: Some(20),
            run_mode: RunMode::Trace,
        };
        let job = sdk.prove_cairo(data).await;
        assert!(job.is_ok());
    }

    #[tokio::test]
    async fn test_unauthorized_access() {
        let unauthorized_key = ProverAccessKey::generate();
        let url = std::env::var("PROVER_URL").unwrap();
        let url = Url::parse(&url).unwrap();
        let sdk = ProverSDK::new(url, unauthorized_key).await.unwrap();
        let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
        let program: CairoCompiledProgram = serde_json::from_str(&program).unwrap();
        let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
        let mut program_input: Vec<Felt> = Vec::new();
        for part in program_input_string.split(',') {
            let felt = Felt::from_dec_str(part).unwrap();
            program_input.push(felt);
        }
        let data = CairoProverInput {
            program,
            layout: Layout::Recursive,
            program_input,
            n_queries: Some(16),
            pow_bits: Some(20),
            run_mode: RunMode::Trace,
        };
        let job = sdk.prove_cairo(data).await;

        assert!(job.is_err());
        // Assert that job is an error and matches the expected error message
        assert!(matches!(job, Err(ProveResponseError(ref msg)) if msg == "Unauthorized"));
    }

    #[tokio::test]
    async fn test_replay_attack_with_same_timestamp() -> Result<(), SdkErrors> {
        // Load program and input
        let program = std::fs::read_to_string("../examples/cairo/fibonacci_compiled.json").unwrap();
        let program: CairoCompiledProgram = serde_json::from_str(&program)?;
        let program_input_string = std::fs::read_to_string("../examples/cairo/input.json").unwrap();
        let mut program_input: Vec<Felt> = Vec::new();
        for part in program_input_string.split(',') {
            let felt = Felt::from_dec_str(part).unwrap();
            program_input.push(felt);
        }

        let data = CairoProverInput {
            program,
            layout: Layout::Recursive,
            program_input,
            n_queries: Some(16),
            pow_bits: Some(20),
            run_mode: RunMode::Trace,
        };
        let data = ProverInput::Cairo(data);

        let private_key = std::env::var("PRIVATE_KEY").unwrap();
        let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
        let signing_key = access_key.0;
        let current_time = Utc::now();
        let current_time_str = current_time.to_rfc3339();
        let too_old_time = current_time - chrono::Duration::minutes(1);
        let too_old_time_str = too_old_time.to_rfc3339();
        let signature = data.sign(signing_key, current_time_str.clone());
        let url = std::env::var("PROVER_URL").unwrap();

        let client = Client::new();

        // First request (Expected: 200 OK)
        let first_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature.clone())
            .header("X-Timestamp", current_time_str.clone())
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            first_response.status(),
            202,
            "First request should be accepted"
        );

        // Second request (Expected: 401 Unauthorized if replay attack is prevented)
        let second_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature.clone())
            .header("X-Timestamp", too_old_time_str.clone())
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            second_response.status(),
            401,
            "Second request with the same timestamp should be rejected"
        );

        Ok(())
    }
}
