use color_eyre::eyre::Result;
use select::document::Document;
use select::predicate::{Name, Predicate};
use states::STATES_BY_NAME;
use std::collections::HashMap;
use std::fs;

fn main() -> Result<()> {
    color_eyre::install()?;
    let html =
        reqwest::blocking::get("https://meric.mo.gov/data/cost-living-data-series")?.text()?;
    let document = Document::from(&*html);
    let table = document.find(Name("table")).next().unwrap();

    let mut cost_ranks = HashMap::with_capacity(50);
    let mut overall = HashMap::with_capacity(50);
    let mut grocery = HashMap::with_capacity(50);
    let mut housing = HashMap::with_capacity(50);
    let mut utilities = HashMap::with_capacity(50);
    let mut transportation = HashMap::with_capacity(50);
    let mut health = HashMap::with_capacity(50);

    for node in table.find(Name("tbody").descendant(Name("tr"))) {
        let mut columns = node.find(Name("td"));
        let state = columns.next().unwrap().text();
        let state_code = match STATES_BY_NAME.get(&*state) {
            Some(state_code) => *state_code,
            None => continue,
        };
        let rank = columns.next().unwrap().text().parse::<usize>().unwrap();
        cost_ranks.insert(state_code, rank);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        overall.insert(state_code, index);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        grocery.insert(state_code, index);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        housing.insert(state_code, index);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        utilities.insert(state_code, index);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        transportation.insert(state_code, index);
        let index = columns.next().unwrap().text().parse::<f32>().unwrap();
        health.insert(state_code, index);
    }
    fs::write(
        "generated/cost_rank.json",
        serde_json::to_string(&cost_ranks)?,
    )?;
    fs::write(
        "generated/cost_index.json",
        serde_json::to_string(&overall)?,
    )?;
    fs::write(
        "generated/grocery_cost_index.json",
        serde_json::to_string(&grocery)?,
    )?;
    fs::write(
        "generated/housing_cost_index.json",
        serde_json::to_string(&housing)?,
    )?;
    fs::write(
        "generated/utilities_cost_index.json",
        serde_json::to_string(&utilities)?,
    )?;
    fs::write(
        "generated/transportation_cost_index.json",
        serde_json::to_string(&transportation)?,
    )?;
    fs::write(
        "generated/health_cost_index.json",
        serde_json::to_string(&health)?,
    )?;
    Ok(())
}
