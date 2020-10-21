#![forbid(unsafe_code)]

use anyhow::Result;
use futures::future::try_join4;

use maryland::{demographic, quality_of_life, taxes, workforce};

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {}", e),
    };
}

async fn run() -> Result<()> {
    try_join4(taxes(), workforce(), demographic(), quality_of_life()).await?;
    Ok(())
}
