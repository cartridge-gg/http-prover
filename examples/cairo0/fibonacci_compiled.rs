let prover_url = context
.prover_url
.clone()
.ok_or(ServerFnError::new("Prover URL not found"))?;

let prover_access_key = context
.prover_access_key
.clone()
.ok_or(ServerFnError::new("Prover access key not found"))?;
let program_path = PathBuf::from(PROGRAM_PATH);

if !program_path.exists() {
return Err(ServerFnError::ServerError(format!(
    "Program file not found: {:?}",
    program_path
)));
}

let access_key = ProverAccessKey::from_hex_string(&prover_access_key)
.map_err(|e| ServerFnError::new(format!("Prover error: {}", e)))?;


let sdk = ProverSDK::new(prover_url.clone(), access_key)
.await
.map_err(|e| ServerFnError::new(format!("Prover error: {}", e)))?;


let program = fs::read_to_string(&program_path)?;
let program_serialized: CairoCompiledProgram = serde_json::from_str(&program)?;

let data = CairoProverInput {
program: program_serialized,
layout: LAYOUT.to_string(),
program_input: chain_events,
pow_bits: Some(POW_BITS),
n_queries: Some(N_QUERIES),
};

log!("Starting proof generation...");
let job_id = sdk.prove_cairo(data).await?;

log!("Proof job created with ID: {:?}", job_id);

sdk.sse(job_id).await?;
let response = sdk.get_job(job_id).await?;
let response = response.text().await?;
let json_response: JobResponse = serde_json::from_str(&response).unwrap();

let prover_result = if let JobResponse::Completed { result, .. } = json_response {
 Ok(result)
}else{
Err(ProverError::CustomError("Job failed".to_string()))
}?;
prover_result