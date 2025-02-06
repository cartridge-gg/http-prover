use std::path::PathBuf;

use clap::Parser;
use prover::utils::config::Template;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct ConfigGenerator {
    #[arg(long, env)]
    pub public_input: PathBuf,
    #[arg(long, env)]
    pub config_file: PathBuf,
}
impl ConfigGenerator {
    pub fn run(self) {
        let config =
            Template::generate_from_public_input_file(&self.public_input, Some(10), Some(10))
                .unwrap();
        println!("{config:#?}");
        config.save_to_file(&self.config_file).unwrap();
    }
}
