use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tokio::fs;

use states::STATES_BY_NAME;

use crate::{parse_percent, BASE_URL};
use futures::future::try_join_all;
use std::str::FromStr;

static PATH: &str = "8mc4-hxm7.json";

#[derive(Debug, Deserialize)]
struct DemoData {
    state: String,
    pop_perc_change_years1: Option<String>,
    median_age: Option<String>,
    population_density: String,
    median_household_income: String,
    percapita_personal_income: String,
    poverty_rate: String,
}

#[derive(Debug)]
struct DemoParsed {
    state: &'static str,
    pop_change: f64,
    median_age: f64,
    pop_density: f64,
    median_household_income: f64,
    percapita_personal_income: f64,
    poverty_rate: f64,
}

impl DemoParsed {
    fn try_from(value: DemoData) -> Result<Self> {
        let state = *STATES_BY_NAME
            .get(value.state.as_str())
            .ok_or_else(|| anyhow!("Invalid state"))?;
        let pop_change = parse_percent(
            &value
                .pop_perc_change_years1
                .ok_or_else(|| anyhow!("Missing pop change"))?,
        )?;
        let median_age = f64::from_str(
            &value
                .median_age
                .ok_or_else(|| anyhow!("Missing median age"))?,
        )?;
        let pop_density = f64::from_str(&value.population_density)?;
        let median_household_income = f64::from_str(&value.median_household_income)?;
        let percapita_personal_income = f64::from_str(&value.percapita_personal_income)?;
        let poverty_rate = parse_percent(&value.poverty_rate)?;
        Ok(DemoParsed {
            state,
            pop_change,
            median_age,
            pop_density,
            median_household_income,
            percapita_personal_income,
            poverty_rate,
        })
    }
}

pub async fn demographic() -> Result<()> {
    let data = get_demographic().await?;
    let pop_change: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.pop_change))
        .collect();
    let median_age: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.median_age))
        .collect();
    let pop_density: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.pop_density))
        .collect();
    let median_household_income: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.median_household_income))
        .collect();
    let percapita_personal_income: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.percapita_personal_income))
        .collect();
    let poverty_rate: HashMap<&str, f64> = data
        .iter()
        .map(|value| (value.state, value.poverty_rate))
        .collect();
    try_join_all(vec![
        fs::write(
            "generated/pop_change.json",
            serde_json::to_string(&pop_change)?,
        ),
        fs::write(
            "generated/median_age.json",
            serde_json::to_string(&median_age)?,
        ),
        fs::write(
            "generated/pop_density.json",
            serde_json::to_string(&pop_density)?,
        ),
        fs::write(
            "generated/median_household_income.json",
            serde_json::to_string(&median_household_income)?,
        ),
        fs::write(
            "generated/percapita_personal_income.json",
            serde_json::to_string(&percapita_personal_income)?,
        ),
        fs::write(
            "generated/poverty_rate.json",
            serde_json::to_string(&poverty_rate)?,
        ),
    ])
    .await?;
    Ok(())
}

async fn get_demographic() -> Result<Vec<DemoParsed>> {
    let uri = format!("{}{}", BASE_URL, PATH);
    Ok(reqwest::get(&uri)
        .await?
        .json::<Vec<DemoData>>()
        .await?
        .into_iter()
        .filter_map(|data| DemoParsed::try_from(data).ok())
        .collect::<Vec<DemoParsed>>())
}
