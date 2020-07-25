#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::env;

use anyhow::{anyhow, Result};
use async_std::fs;
use dotenv::dotenv;
use serde::Deserialize;

use states::STATES;

#[derive(Debug, Deserialize)]
struct Park {
    states: String,
    #[serde(rename = "parkCode")]
    park_code: String,
    designation: String,
}

impl Park {
    fn get_states(&self) -> Vec<&str> {
        self.states.split(',').collect()
    }
}

#[derive(Debug, Deserialize)]
struct Category {
    name: String,
    parks: Vec<Park>,
}

#[derive(Debug, Deserialize)]
struct Response {
    data: Vec<Category>,
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
    let categories = get_categories().await?;
    let parks: Vec<Park> = categories
        .into_iter()
        .flat_map(|category| category.parks)
        .collect();
    let mut parks_by_state: HashMap<&str, HashSet<&str>> = STATES
        .iter()
        .map(|state| (*state, HashSet::new()))
        .collect();
    for park in parks.iter() {
        let park_code = park.park_code.as_str();
        for state in park.get_states() {
            if let Some(park_set) = parks_by_state.get_mut(state) {
                park_set.insert(park_code);
            }
        }
    }
    let result: HashMap<&'static str, usize> = parks_by_state
        .into_iter()
        .map(|(state, parks)| (state, parks.len()))
        .collect();
    fs::write(
        "generated/national_parks.json",
        serde_json::to_string(&result)?,
    )
    .await?;
    Ok(())
}

async fn get_categories() -> Result<Vec<Category>> {
    let api_key = env::var("NPS_KEY")?;
    let uri = format!(
        "https://developer.nps.gov/api/v1/activities/parks?api_key={}",
        api_key
    );
    let response: Response = surf::get(uri)
        .set_header("accept", "application/json")
        .recv_json()
        .await
        .map_err(|e| anyhow!("Could not fetch data: {}", e))?;
    Ok(response.data)
}

// curl -X GET "" -H  "accept: application/json"
