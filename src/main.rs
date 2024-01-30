
use tokio::io::{self, BufReader};

mod cli {
    pub mod commands;
    pub mod memory_analysis;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let  _stdin = BufReader::new(io::stdin());

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("check") => cli::commands::check::execute(),
        Some("details") => cli::commands::details::execute().await,
        Some("optimize") => cli::commands::optimize::execute(),
        _ => println!("Invalid command or command not provided"),
    };

    Ok(())
    ??
}
