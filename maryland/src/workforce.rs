use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tokio::fs;

use states::STATES_BY_NAME;

use crate::{parse_percent, BASE_URL};

static PATH: &str = "5esm-neyf.json";

#[derive(Debug, Deserialize)]
struct WorkforceData {
    state: String,
    unemployment_rate: Option<String>,
}

#[derive(Debug)]
struct WorkforceParsed {
    state: &'static str,
    unemployment: f64,
}

impl WorkforceParsed {
    fn try_from(value: WorkforceData) -> Result<Self> {
        let state = *STATES_BY_NAME
            .get(value.state.as_str())
            .ok_or_else(|| anyhow!("Invalid state"))?;
        let unemployment = parse_percent(
            &value
                .unemployment_rate
                .ok_or_else(|| anyhow!("Missing unemployment rate"))?,
        )?;
        Ok(WorkforceParsed {
            state,
            unemployment,
        })
    }
}

pub async fn workforce() -> Result<()> {
    let data = get_workforce().await?;
    let unemployment: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.unemployment))
        .collect();
    fs::write(
        "generated/unemployment.json",
        serde_json::to_string(&unemployment)?,
    )
    .await?;
    Ok(())
}

async fn get_workforce() -> Result<Vec<WorkforceParsed>> {
    let uri = format!("{}{}", BASE_URL, PATH);
    Ok(reqwest::get(&uri)
        .await?
        .json::<Vec<WorkforceData>>()
        .await?
        .into_iter()
        .filter_map(|data| WorkforceParsed::try_from(data).ok())
        .collect::<Vec<WorkforceParsed>>())
}
