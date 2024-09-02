mod template;
mod blockchain;
mod utils;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8000, help = "port for running the node")]
    port: u32,
}

fn main() {
    blockchain::blockchain_app::blockchain_app(Args::parse());
}