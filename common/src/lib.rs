pub mod cairo0_prover_input;
pub mod cairo_prover_input;


pub trait ProverInput {
    fn serialize(self) -> serde_json::Value;
}
