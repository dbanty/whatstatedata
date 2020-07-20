mod states;

use anyhow::Result;
use async_std::{fs, task};

use serde::{Deserialize, Serialize};
use crate::states::STATES_BY_NAME;
use std::collections::HashMap;

// https://worldpopulationreview.com/ab308290-e944-4e4f-b7db-0b2a4c525d1f"
// CSV with State medianPropertyTax Pop
// Output two files - property_tax.json and population.json
// Each is map of state to value
// TODO: download data from source instead of storing here

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "medianPropertyTax")]
    tax_rate: f32,
    #[serde(rename = "Pop")]
    population: u64,
}

#[derive(Debug, Serialize)]
struct DataSource {
    source: &'static str,
    name: &'static str,
    data_type: &'static str,
}

static PROPERTY_TAX: DataSource = DataSource {
    source: "property_taxes.json",
    name: "Property Taxes",
    data_type: "percent",
};

static POPULATION: DataSource = DataSource {
    source: "populations.json",
    name: "Population",
    data_type: "number",
};

async fn pop_and_taxes() -> Result<[&'static DataSource; 2]> {
    let contents = fs::read("data.csv").await?.into_boxed_slice();
    let mut rdr = csv::Reader::from_reader(&*contents);
    let mut taxes = HashMap::<&str, f32>::new();
    let mut populations = HashMap::<&str, u64>::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        if let Some(state) = STATES_BY_NAME.get(record.state.as_str()) {
            taxes.insert(state, record.tax_rate);
            populations.insert(state, record.population);
        }
    }
    fs::write(PROPERTY_TAX.source, serde_json::to_string(&taxes)?).await?;
    fs::write(POPULATION.source, serde_json::to_string(&populations)?).await?;
    Ok([&PROPERTY_TAX, &POPULATION])
}

fn main() {
    let pop_and_taxes_fut = pop_and_taxes();
    if let Ok(result) = task::block_on(pop_and_taxes_fut) {
        if let Ok(json) = serde_json::to_string(&result) {
            task::block_on(fs::write("sources.json", json));
        }
    }
}
