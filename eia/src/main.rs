#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::{env, fs};

use anyhow::{anyhow, Result};
use dotenv::dotenv;
use futures::future::try_join_all;
use serde::Deserialize;

use states::STATES;

#[derive(Debug, Deserialize)]
struct SeriesData {
    data: [(String, f64); 1],
}

#[derive(Debug, Deserialize)]
struct Response {
    series: [SeriesData; 1],
}

#[async_std::main]
async fn main() {
    dotenv().ok();
    match run().await {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {}", e),
    };
}

async fn run() -> Result<()> {
    let api_key = env::var("EIA_KEY")?;
    let futures = STATES.iter().map(|state| get_emissions(&api_key, state));
    let result: HashMap<&'static str, f64> = try_join_all(futures).await?.into_iter().collect();
    fs::write(
        "generated/co2_emissions.json",
        serde_json::to_string(&result)?,
    )?;
    Ok(())
}

async fn get_emissions(api_key: &str, state: &'static str) -> Result<(&'static str, f64)> {
    let uri = format!(
        "https://api.eia.gov/series/?api_key={}&series_id=EMISS.CO2-TOTV-TT-TO-{}.A&start=2017",
        api_key, state
    );
    let mut response = surf::get(uri)
        .await
        .map_err(|e| anyhow!("Could not fetch data: {}", e))?;
    let json_body: Response = response
        .body_json()
        .await
        .map_err(|e| anyhow!("Could not parse JSON {}", e))?;
    Ok((state, json_body.series[0].data[0].1))
}
