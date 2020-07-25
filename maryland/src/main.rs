#![forbid(unsafe_code)]

use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};
use async_std::fs;
use serde::Deserialize;

use serde::export::TryFrom;
use states::STATES_BY_NAME;
use std::str::FromStr;

static BASE_URL: &str = "https://opendata.maryland.gov/resource/";
static TAXES: &str = "t833-r94z.json";

#[derive(Debug, Deserialize)]
struct TaxData {
    state: String,
    state_individual_income_taxrate: String,
    state_corporate_income_taxrate: String,
    state_sales_taxrate: String,
}

#[derive(Debug)]
struct TaxDataParsed {
    state: &'static str,
    income_tax: f64,
    corp_income_tax: f64,
    sales_tax: f64,
}

fn parse_percent(val: &str) -> Result<f64> {
    let no_percent = if val.ends_with('%') {
        &val[..val.len() - 1]
    } else {
        val
    };
    f64::from_str(no_percent)
        .map(|f| f / 100.0)
        .map_err(|e| anyhow!(e))
}

impl TryFrom<TaxData> for TaxDataParsed {
    type Error = Error;

    fn try_from(value: TaxData) -> Result<Self, Self::Error> {
        let state = *STATES_BY_NAME
            .get(value.state.as_str())
            .ok_or_else(|| anyhow!("Invalid state"))?;
        let income_tax = parse_percent(&value.state_individual_income_taxrate)?;
        let corp_income_tax = parse_percent(&value.state_corporate_income_taxrate)?;
        let sales_tax = parse_percent(&value.state_sales_taxrate)?;
        Ok(TaxDataParsed {
            state,
            income_tax,
            corp_income_tax,
            sales_tax,
        })
    }
}

#[async_std::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Error: {}", e),
    };
}

async fn run() -> Result<()> {
    taxes().await?;
    Ok(())
}

async fn taxes() -> Result<()> {
    let data = get_taxes().await?;
    let income_tax: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.income_tax))
        .collect();
    let corp_tax: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.corp_income_tax))
        .collect();
    let sales_tax: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.sales_tax))
        .collect();
    futures::future::try_join3(
        fs::write(
            "generated/income_tax.json",
            serde_json::to_string(&income_tax)?,
        ),
        fs::write(
            "generated/corporate_income_tax.json",
            serde_json::to_string(&corp_tax)?,
        ),
        fs::write(
            "generated/sales_tax.json",
            serde_json::to_string(&sales_tax)?,
        ),
    )
    .await?;
    Ok(())
}

async fn get_taxes() -> Result<Vec<TaxDataParsed>> {
    let uri = format!("{}{}", BASE_URL, TAXES);
    Ok(surf::get(uri)
        .recv_json::<Vec<TaxData>>()
        .await
        .map_err(|e| anyhow!("Could not fetch data: {}", e))?
        .into_iter()
        .filter_map(|data| TaxDataParsed::try_from(data).ok())
        .collect::<Vec<TaxDataParsed>>())
}
