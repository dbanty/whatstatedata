use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tokio::fs;

use states::STATES_BY_NAME;

use crate::{parse_percent, BASE_URL};

static PATH: &str = "cz6x-aq2i.json";

#[derive(Debug, Deserialize)]
struct QOLData {
    state: String,
    broadband_internet: Option<String>,
}

#[derive(Debug)]
struct QOLParsed {
    state: &'static str,
    broadband_internet: f64,
}

impl QOLParsed {
    fn try_from(value: QOLData) -> Result<Self> {
        let state = *STATES_BY_NAME
            .get(value.state.as_str())
            .ok_or_else(|| anyhow!("Invalid state"))?;
        let broadband_internet = parse_percent(
            &value
                .broadband_internet
                .ok_or_else(|| anyhow!("Missing unemployment rate"))?,
        )?;
        Ok(QOLParsed {
            state,
            broadband_internet,
        })
    }
}

pub async fn quality_of_life() -> Result<()> {
    let data = get_qol().await?;
    let broadband_internet: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.broadband_internet))
        .collect();
    fs::write(
        "generated/broadband_internet.json",
        serde_json::to_string(&broadband_internet)?,
    )
    .await?;
    Ok(())
}

async fn get_qol() -> Result<Vec<QOLParsed>> {
    let uri = format!("{}{}", BASE_URL, PATH);
    Ok(reqwest::get(&uri)
        .await?
        .json::<Vec<QOLData>>()
        .await?
        .into_iter()
        .filter_map(|data| QOLParsed::try_from(data).ok())
        .collect::<Vec<QOLParsed>>())
}
