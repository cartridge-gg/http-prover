use crate::errors::ProveErrors;
use crate::Args;
use crate::CairoVersion;
use common::{
    cairo0_prover_input::{Cairo0CompiledProgram, Cairo0ProverInput},
    cairo_prover_input::{CairoCompiledProgram, CairoProverInput},
};
use prover_sdk::sdk::ProverSDK;

pub async fn prove(args: Args, sdk: ProverSDK) -> Result<String, ProveErrors> {
    let program = std::fs::read_to_string(&args.program_path)?;
    let proof = match args.cairo_version {
        CairoVersion::V0 => {
            let program_serialized: Cairo0CompiledProgram = serde_json::from_str(&program)?;
            let data = Cairo0ProverInput {
                program: program_serialized,
                layout: args.layout,
                program_input_path: args.program_input_path.unwrap(),
            };
            sdk.prove_cairo0(data).await?
        }
        CairoVersion::V1 => {
            let program_serialized: CairoCompiledProgram = serde_json::from_str(&program)?;
            let data = CairoProverInput {
                program: program_serialized,
                layout: args.layout,
                program_input_path: args.program_input_path.unwrap(),
            };
            sdk.prove_cairo(data).await?
        }
    };
    Ok(proof)
}
//TODO: Add support for putting arguments in the program_input field
