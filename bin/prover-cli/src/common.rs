use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

#[derive(Debug, Serialize, Deserialize, ValueEnum, Clone)]
pub enum CairoVersion {
    V0,
    V1,
}

pub fn validate_input(input: &str) -> Vec<Felt> {
    let parts: Vec<&str> = input.split(',').collect();

    let mut felts = Vec::new();
    for part in parts {
        let part = part.replace(['[', '\n', ']'], "");
        felts.push(part.trim().parse::<Felt>().unwrap());
    }
    felts
}
