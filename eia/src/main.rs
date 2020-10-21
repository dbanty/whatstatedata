#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::env;

use anyhow::Result;
use dotenv::dotenv;
use futures::future::{try_join, try_join_all};
use reqwest::Client;
use serde::Deserialize;
use tokio::fs;

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    match run().await {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {}", e),
    };
}

async fn run() -> Result<()> {
    let api_key = env::var("EIA_KEY")?;
    let client = Client::new();
    try_join(
        co2_emissions(&api_key, &client),
        consumption(&api_key, &client),
    )
    .await?;
    Ok(())
}

async fn co2_emissions(api_key: &str, client: &Client) -> Result<()> {
    let futures = STATES
        .iter()
        .map(|state| get_emissions(api_key, state, client));
    let result: HashMap<&'static str, f64> = try_join_all(futures).await?.into_iter().collect();
    fs::write(
        "generated/co2_emissions.json",
        serde_json::to_string(&result)?,
    )
    .await?;
    Ok(())
}

async fn get_emissions<'a>(
    api_key: &str,
    state: &'a str,
    client: &Client,
) -> Result<(&'a str, f64)> {
    let uri = format!(
        "https://api.eia.gov/series/?api_key={}&series_id=EMISS.CO2-TOTV-TT-TO-{}.A&start=2017",
        api_key, state
    );
    let json_body = client.get(&uri).send().await?.json::<Response>().await?;
    Ok((state, json_body.get_value()))
}

async fn consumption(api_key: &str, client: &Client) -> Result<()> {
    let futures = STATES
        .iter()
        .map(|state| get_consumption(&api_key, state, client));
    let result: HashMap<&'static str, f64> = try_join_all(futures).await?.into_iter().collect();
    fs::write(
        "generated/percent_renewable.json",
        serde_json::to_string(&result)?,
    )
    .await?;
    Ok(())
}

async fn get_consumption<'a>(
    api_key: &str,
    state: &'a str,
    client: &Client,
) -> Result<(&'a str, f64)> {
    let total_uri = format!(
        "http://api.eia.gov/series/?api_key={}&series_id=SEDS.TETCB.{}.A&start=2018",
        api_key, state
    );
    let renewable_uri = format!(
        "http://api.eia.gov/series/?api_key={}&series_id=SEDS.RETCB.{}.A&start=2018",
        api_key, state
    );
    let total_future = client.get(&total_uri).send();
    let renewable_future = client.get(&renewable_uri).send();
    let (total_response, renewable_response) = try_join(total_future, renewable_future).await?;
    let (total_data, renewable_data) = try_join(
        total_response.json::<Response>(),
        renewable_response.json::<Response>(),
    )
    .await?;

    let percent_renewable = renewable_data.get_value() / total_data.get_value();
    Ok((state, percent_renewable))
}
