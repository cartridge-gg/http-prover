use itertools::{chain, Itertools};
use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use swiftness_proof_parser::{json_parser, stark_proof, StarkProof};
use transform::StarkProofExprs;
use vec_felt::{VecFelt, VecFeltError};
pub mod transform;
pub mod vec_felt;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofParserError {
    #[error(transparent)]
    VecFelt(#[from] VecFeltError),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    InternalParserError(#[from] anyhow::Error),
}
pub fn extract_program_hash(stark_proof: StarkProof) -> Felt {
    let program_output_range = &stark_proof.public_input.segments[2];
    let main_page_len = stark_proof.public_input.main_page.len();
    let output_len = (program_output_range.stop_ptr - program_output_range.begin_addr) as usize;
    let program = stark_proof.public_input.main_page[0..main_page_len - output_len].to_vec();

    let values: Vec<Felt> = program
        .iter()
        .map(|el| {
            let number = &el.value;

            let mut padded_bytes = [0u8; 32];
            let bytes = number.to_bytes_be();

            let bytes_len = bytes.len();

            padded_bytes[32 - bytes_len..].copy_from_slice(&bytes);

            Felt::from_bytes_be(&padded_bytes)
        })
        .collect();
    poseidon_hash_many(&values)
}
pub fn extract_program_output(stark_proof: StarkProof) -> Vec<Felt> {
    let program_output_range = &stark_proof.public_input.segments[2];
    let main_page_len = stark_proof.public_input.main_page.len();
    let output_len = (program_output_range.stop_ptr - program_output_range.begin_addr) as usize;
    let program_output = stark_proof.public_input.main_page[main_page_len - output_len..].to_vec();
    let values: Vec<Felt> = program_output
        .iter()
        .map(|el| {
            let number = &el.value;

            let mut padded_bytes = [0u8; 32];
            let bytes = number.to_bytes_be();

            let bytes_len = bytes.len();

            padded_bytes[32 - bytes_len..].copy_from_slice(&bytes);

            Felt::from_bytes_be(&padded_bytes)
        })
        .collect();
    values
}
pub fn program_output_hash(felts: Vec<Felt>) -> Felt {
    poseidon_hash_many(&felts)
}
pub fn parse_proof(proof: String) -> Result<StarkProof, ProofParserError> {
    let proof_json = serde_json::from_str::<json_parser::StarkProof>(&proof)?;
    Ok(stark_proof::StarkProof::try_from(proof_json)?)
}

pub fn proof_to_felt(proof: StarkProof) -> Result<Vec<Felt>, ProofParserError> {
    let stark_proof: StarkProofExprs = proof.into();
    let config: VecFelt = serde_json::from_str(&stark_proof.config.to_string())?;
    let public_input: VecFelt = serde_json::from_str(&stark_proof.public_input.to_string())?;
    let unsent_commitment: VecFelt =
        serde_json::from_str(&stark_proof.unsent_commitment.to_string())?;
    let witness: VecFelt = serde_json::from_str(&stark_proof.witness.to_string())?;

    Ok(chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    )
    .collect_vec())
}

#[cfg(test)]
mod tests {
    use crate::{
        extract_program_hash, extract_program_output, parse_proof, program_output_hash,
        proof_to_felt,
    };

    pub fn read_proof() -> String {
        let proof = std::fs::read_to_string("example/proof.json").unwrap();
        proof
    }
    #[test]
    fn test_extract_program_hash() {
        let proof = read_proof();
        let stark_proof = parse_proof(proof);
        let program_hash = extract_program_hash(stark_proof.unwrap());
        assert_eq!(
            program_hash.to_string(),
            "2251972324230578422543092394494031242690791181195034520556584290316798249271"
        );
    }
    #[test]
    fn test_extract_program_output() {
        let proof = read_proof();
        let stark_proof = parse_proof(proof);
        let program_output = extract_program_output(stark_proof.unwrap());
        assert_eq!(program_output.len(), 7);
        assert_eq!("0", program_output[0].to_string());
        assert_eq!("1", program_output[1].to_string());
        assert_eq!("1", program_output[2].to_string());
        assert_eq!("3", program_output[3].to_string());
        assert_eq!("1", program_output[4].to_string());
        assert_eq!("2", program_output[5].to_string());
        assert_eq!("3", program_output[6].to_string());
    }
    #[test]
    fn test_program_output_hash() {
        let proof = read_proof();
        let stark_proof = parse_proof(proof);
        let program_output = extract_program_output(stark_proof.unwrap());
        let hash = program_output_hash(program_output);
        assert_eq!(
            hash.to_string(),
            "2144555888719052742880342011775786530333616377198088482005787934731079204155"
        );
    }
    #[test]
    fn test_parse_proof() {
        let proof = read_proof();
        let stark_proof = parse_proof(proof);
        let proof = proof_to_felt(stark_proof.unwrap());
        assert!(proof.is_ok());
        assert_eq!(proof.unwrap().len(), 2533);
    }
}
