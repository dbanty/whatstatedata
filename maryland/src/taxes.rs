use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_std::fs;
use futures::future::try_join3;
use serde::Deserialize;

use states::STATES_BY_NAME;

use crate::{parse_percent, BASE_URL};

static PATH: &str = "t833-r94z.json";

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

impl TaxDataParsed {
    fn try_from(value: TaxData) -> Result<Self> {
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

pub async fn taxes() -> Result<()> {
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
    try_join3(
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
    let uri = format!("{}{}", BASE_URL, PATH);
    Ok(surf::get(uri)
        .recv_json::<Vec<TaxData>>()
        .await
        .map_err(|e| anyhow!("Could not fetch tax data: {}", e))?
        .into_iter()
        .filter_map(|data| TaxDataParsed::try_from(data).ok())
        .collect::<Vec<TaxDataParsed>>())
}
