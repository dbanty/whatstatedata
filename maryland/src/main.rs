#![forbid(unsafe_code)]

use anyhow::Result;
use futures::future::try_join3;

use maryland::{demographic, taxes, workforce};

#[async_std::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {}", e),
    };
}

async fn run() -> Result<()> {
    try_join3(taxes(), workforce(), demographic()).await?;
    Ok(())
}
