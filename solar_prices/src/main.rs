#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs::File;
use std::{fs, thread};

use anyhow::{anyhow, Result};
use serde::Deserialize;
use time::Date;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "System Size")]
    size: f64,
    #[serde(rename = "Total Installed Price")]
    price: f64,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "Installation Date")]
    #[serde(with = "parse_date")]
    date: Date,
}

mod parse_date {
    use serde::{self, Deserialize, Deserializer};
    use time::Date;

    const FORMAT: &str = "%-m/%-d/%Y";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

fn load_data(path: &str) -> Result<HashMap<&str, Vec<f64>>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut prices: HashMap<&str, Vec<f64>> = states::STATES
        .iter()
        .map(|state| (*state, Vec::new()))
        .collect();
    for result in rdr.deserialize() {
        if result.is_err() {
            continue;
        }
        let record: Record = result?;
        #[allow(clippy::float_cmp)]
        if record.price == 0.0 || record.price == -9999.0 {
            continue;
        }
        #[allow(clippy::float_cmp)]
        if record.size == 0.0 || record.size == -9999.0 {
            continue;
        }
        let price = record.price / record.size;
        prices
            .get_mut(record.state.as_str())
            .ok_or_else(|| anyhow!("Missing state {}!", record.state))?
            .push(price);
    }
    Ok(prices)
}

fn run() -> Result<()> {
    let part_1 = thread::spawn(|| load_data("raw_data/tracking-the-sun/part_1.csv"));
    let part_2 = thread::spawn(|| load_data("raw_data/tracking-the-sun/part_2.csv"));
    let part_1_prices = part_1.join().unwrap()?;
    let part_2_prices = part_2.join().unwrap()?;
    let mut averages: HashMap<&str, Option<f64>> = HashMap::with_capacity(states::STATES.len());
    for state in states::STATES.iter() {
        let part_1_data = part_1_prices
            .get(state)
            .ok_or_else(|| anyhow!("part_1 was missing state {} ", state))?;
        let part_2_data = part_2_prices
            .get(state)
            .ok_or_else(|| anyhow!("part_2 was missing state {} ", state))?;
        let sum: f64 = part_1_data.iter().sum::<f64>() + part_2_data.iter().sum::<f64>();
        let len = part_1_data.len() + part_2_data.len();
        let mut average = None;
        if len > 0 {
            average = Some(sum / len as f64);
        }
        averages.insert(state, average);
    }
    fs::write(
        "generated/solar_prices.json",
        serde_json::to_string(&averages)?,
    )?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("ERROR: {}", e),
    }
}
