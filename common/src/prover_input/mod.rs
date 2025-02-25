mod cairo;
mod cairo0;

use std::{fmt::Display, str::FromStr};

pub use cairo::{CairoCompiledProgram, CairoProverInput};
pub use cairo0::LayoutBridgeInput;
pub use cairo0::{Cairo0CompiledProgram, Cairo0ProverInput};
use clap::ValueEnum;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::{sign_data, HttpProverData};

#[derive(Debug)]
pub enum ProverInput {
    Cairo0(Cairo0ProverInput),
    Cairo(CairoProverInput),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum RunMode {
    Bootload,
    Pie,
    Trace,
}

impl HttpProverData for ProverInput {
    fn to_json_value(&self) -> serde_json::Value {
        match self {
            ProverInput::Cairo0(input) => serde_json::to_value(input).unwrap(),
            ProverInput::Cairo(input) => serde_json::to_value(input).unwrap(),
        }
    }
    fn sign(&self, signing_key: SigningKey, timestamp: String, nonce: u64) -> String {
        match self {
            ProverInput::Cairo0(input) => sign_data(input, &timestamp, &signing_key, nonce),
            ProverInput::Cairo(input) => sign_data(input, &timestamp, &signing_key, nonce),
        }
    }
}

impl HttpProverData for LayoutBridgeInput {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn sign(&self, signing_key: SigningKey, timestamp: String, nonce: u64) -> String {
        sign_data(self, &timestamp, &signing_key, nonce)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Layout {
    Small,
    Dex,
    Recursive,
    RecursiveWithPoseidon,
    Starknet,
    StarknetWithKeccak,
}
impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Small => write!(f, "small"),
            Layout::Dex => write!(f, "dex"),
            Layout::Recursive => write!(f, "recursive"),
            Layout::RecursiveWithPoseidon => write!(f, "recursive_with_poseidon"),
            Layout::Starknet => write!(f, "starknet"),
            Layout::StarknetWithKeccak => write!(f, "starknet_with_keccak"),
        }
    }
}
impl FromStr for Layout {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "small" => Ok(Layout::Small),
            "dex" => Ok(Layout::Dex),
            "recursive" => Ok(Layout::Recursive),
            "recursive_with_poseidon" => Ok(Layout::RecursiveWithPoseidon),
            "starknet" => Ok(Layout::Starknet),
            "starknet_with_keccak" => Ok(Layout::StarknetWithKeccak),
            _ => Err(format!("Invalid layout: {}", s)),
        }
    }
}
impl Layout {
    pub fn is_bootloadable(&self) -> bool {
        matches!(
            self,
            Layout::Recursive
                | Layout::RecursiveWithPoseidon
                | Layout::Starknet
                | Layout::StarknetWithKeccak
        )
    }
}
