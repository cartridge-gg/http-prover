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
    async fn test_replay_attack_with_same_nonce() -> Result<(), SdkErrors> {
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
        let url = std::env::var("PROVER_URL").unwrap();

        let client = Client::new();

        // Generate a unique nonce
        let nonce: u64 = rand::random(); // Ensures a fresh nonce

        let signature = data.sign(signing_key.clone(), current_time_str.clone(), nonce);

        // First request (Expected: 202 Accepted)
        let first_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature.clone())
            .header("X-Timestamp", current_time_str.clone())
            .header("X-Nonce", nonce.to_string()) // Include nonce
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            first_response.status(),
            202,
            "First request should be accepted"
        );

        // Second request with the same nonce (Expected: 401 Unauthorized)
        let second_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature.clone())
            .header("X-Timestamp", current_time_str.clone()) // Same timestamp
            .header("X-Nonce", nonce.to_string()) // Same nonce (should be rejected)
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            second_response.status(),
            401,
            "Second request with the same nonce should be rejected"
        );

        // Third request with a new nonce (Expected: 202 Accepted)
        let new_nonce: u64 = rand::random(); // Generate a new nonce
        let new_signature = data.sign(signing_key, current_time_str.clone(), new_nonce);

        let third_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", new_signature)
            .header("X-Timestamp", current_time_str.clone())
            .header("X-Nonce", new_nonce.to_string()) // New nonce (should be accepted)
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            third_response.status(),
            202,
            "Third request with a new nonce should be accepted"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_different_nonce_same_timestamp() -> Result<(), SdkErrors> {
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
        let url = std::env::var("PROVER_URL").unwrap();

        let client = Client::new();

        // Generate first nonce
        let nonce_a: u64 = rand::random();
        let signature_a = data.sign(signing_key.clone(), current_time_str.clone(), nonce_a);

        // First request (Expected: 202 Accepted)
        let first_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature_a.clone())
            .header("X-Timestamp", current_time_str.clone())
            .header("X-Nonce", nonce_a.to_string()) // Unique nonce
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            first_response.status(),
            202,
            "First request should be accepted"
        );

        // Generate second nonce (different, but timestamp stays the same)
        let nonce_b: u64 = rand::random();
        let signature_b = data.sign(signing_key, current_time_str.clone(), nonce_b);

        // Second request (Expected: 202 Accepted)
        let second_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature_b.clone())
            .header("X-Timestamp", current_time_str.clone()) // Same timestamp
            .header("X-Nonce", nonce_b.to_string()) // Different nonce
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            second_response.status(),
            202,
            "Second request with a different nonce but same timestamp should be accepted"
        );

        Ok(())
    }
    #[tokio::test]
    async fn test_outdated_timestamp_with_different_nonce() -> Result<(), SdkErrors> {
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

        // Outdated timestamp (older than 30 seconds)
        let outdated_time = current_time - chrono::Duration::seconds(31);
        let outdated_time_str = outdated_time.to_rfc3339();

        let url = std::env::var("PROVER_URL").unwrap();
        let client = Client::new();

        // Generate first nonce
        let nonce_a: u64 = rand::random();
        let signature_a = data.sign(signing_key.clone(), current_time_str.clone(), nonce_a);

        // First request (Expected: 202 Accepted)
        let first_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature_a.clone())
            .header("X-Timestamp", current_time_str.clone()) // Valid timestamp
            .header("X-Nonce", nonce_a.to_string()) // Unique nonce
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            first_response.status(),
            202,
            "First request should be accepted"
        );

        // Generate second nonce (different, but with outdated timestamp)
        let nonce_b: u64 = rand::random();
        let signature_b = data.sign(signing_key, outdated_time_str.clone(), nonce_b);

        // Second request (Expected: 401 Unauthorized due to outdated timestamp)
        let second_response = client
            .post(format!("{}/prove/cairo", url.clone()))
            .header("X-Signature", signature_b.clone())
            .header("X-Timestamp", outdated_time_str.clone()) // Outdated timestamp
            .header("X-Nonce", nonce_b.to_string()) // Different nonce
            .json(&data.to_json_value())
            .send()
            .await?;

        assert_eq!(
            second_response.status(),
            401,
            "Second request with an outdated timestamp should be rejected"
        );

        Ok(())
    }
}
