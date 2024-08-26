use cairo_prove::Args;
use clap::Parser;

#[tokio::main]
pub async fn main(){
    let args = Args::parse();
    cairo_prove::prove(args).await.unwrap();
}