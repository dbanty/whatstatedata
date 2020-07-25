#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::env;

use anyhow::{anyhow, Result};
use async_std::fs;
use dotenv::dotenv;
use futures::future::{try_join, try_join_all};
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

impl Response {
    fn get_value(&self) -> f64 {
        self.series[0].data[0].1
    }
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
    try_join(co2_emissions(&api_key), consumption(&api_key)).await?;
    Ok(())
}

async fn co2_emissions(api_key: &str) -> Result<()> {
    let futures = STATES.iter().map(|state| get_emissions(&api_key, state));
    let result: HashMap<&'static str, f64> = try_join_all(futures).await?.into_iter().collect();
    fs::write(
        "generated/co2_emissions.json",
        serde_json::to_string(&result)?,
    )
    .await?;
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
    Ok((state, json_body.get_value()))
}

async fn consumption(api_key: &str) -> Result<()> {
    let futures = STATES.iter().map(|state| get_consumption(&api_key, state));
    let result: HashMap<&'static str, f64> = try_join_all(futures).await?.into_iter().collect();
    fs::write(
        "generated/percent_renewable.json",
        serde_json::to_string(&result)?,
    )
    .await?;
    Ok(())
}

async fn get_consumption(api_key: &str, state: &'static str) -> Result<(&'static str, f64)> {
    let total_uri = format!(
        "http://api.eia.gov/series/?api_key={}&series_id=SEDS.TETCB.{}.A&start=2018",
        api_key, state
    );
    let renewable_uri = format!(
        "http://api.eia.gov/series/?api_key={}&series_id=SEDS.RETCB.{}.A&start=2018",
        api_key, state
    );
    let total_future = surf::get(total_uri).recv_json::<Response>();
    let renewable_future = surf::get(renewable_uri).recv_json::<Response>();
    let (total_data, renewable_data) = try_join(total_future, renewable_future)
        .await
        .map_err(|e| anyhow!("Error getting consumption: {}", e))?;

    let percent_renewable = renewable_data.get_value() / total_data.get_value();
    Ok((state, percent_renewable))
}
