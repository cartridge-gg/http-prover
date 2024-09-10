mod cairo;
mod cairo0;

pub use cairo::{CairoCompiledProgram, CairoProverInput};
pub use cairo0::{Cairo0CompiledProgram, Cairo0ProverInput};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ProverInput {
    Cairo0(Cairo0ProverInput),
    Cairo(CairoProverInput),
}
