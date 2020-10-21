use std::collections::HashMap;
use std::env;

use backoff::{future::FutureOperation as _, Error, ExponentialBackoff};
use dotenv::dotenv;
use eyre::{eyre, Result};
use futures::future::try_join_all;
use serde::Deserialize;
use serde_json;
use tokio::fs;

use reqwest::Client;
use states::STATES_BY_NAME;
use std::fmt::Debug;

const STATE_IDS_PATH: &str = "raw_data/noaa_states.json";

type Code = String;
type ID = String;

/// Attempt to load states from a file (for caching web request results)
async fn read_states_from_file() -> Result<HashMap<ID, Code>> {
    Ok(serde_json::from_str(
        &*fs::read_to_string(STATE_IDS_PATH).await?,
    )?)
}

const GET_STATES_URI: &str =
    "https://www.ncdc.noaa.gov/cdo-web/api/v2/locations?datasetid=NORMAL_ANN&locationcategoryid=ST&limit=51";

#[derive(Debug, Deserialize)]
struct StateData {
    name: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct Data {
    value: f64,
}

#[derive(Debug, Deserialize)]
struct GetResponse<T> {
    results: Vec<T>,
}

async fn read_states_from_web(token: &str, client: &Client) -> Result<HashMap<ID, Code>> {
    Ok(client
        .get(GET_STATES_URI)
        .header("token", token)
        .send()
        .await?
        .json::<GetResponse<StateData>>()
        .await?
        .results
        .into_iter()
        .filter_map(|state| {
            Some((
                state.id.to_owned(),
                STATES_BY_NAME.get(&*state.name)?.to_owned().to_owned(),
            ))
        })
        .collect::<HashMap<ID, Code>>())
}

/// Gets states IDs as needed by NOAA. Will load from raw_data if available, or fetch from
/// NOAA's API if missing.
async fn get_states(token: &str, client: &Client) -> Result<HashMap<ID, Code>> {
    if let Ok(states) = read_states_from_file().await {
        return Ok(states);
    }
    let states = read_states_from_web(token, client).await?;
    fs::write(STATE_IDS_PATH, serde_json::to_string(&states)?).await?;
    Ok(states)
}

const DATA_URL: &str = "https://www.ncdc.noaa.gov/cdo-web/api/v2/data?datasetid=NORMAL_ANN&startdate=2000-01-01&enddate=2010-01-01&units=standard&limit=1000&includemetadata=false";

/// Request data for a specific type and state from NOAA's API
async fn data_request<'a>(
    token: &str,
    state_id: &'a ID,
    data_type: &str,
    client: &reqwest::Client,
) -> Result<String, Error<eyre::Error>> {
    let err_mapper = |e| {
        Error::Permanent(eyre::eyre!(
            "Encountered {:#?} when fetching data type {} for state {}",
            e,
            data_type,
            state_id
        ))
    };

    let response = client
        .get(&format!(
            "{}&locationid={}&datatypeid={}",
            DATA_URL, state_id, data_type
        ))
        .header("token", token)
        .send()
        .await
        .map_err(err_mapper)?;
    return if response.status() == 429 {
        Err(Error::Transient(eyre!("Too many requests")))
    } else {
        Ok(response.text().await.map_err(err_mapper)?)
    };
}

/// Attempt to load a cached value from a file
async fn load_data_from_file(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path).await?)
}

/// Get the value of a specific data type for a specific state. Return (state_id, value)
async fn get_data_for_state<'a>(
    token: &str,
    state_id: &'a ID,
    data_type: &str,
    client: &Client,
) -> Result<(&'a ID, f64)> {
    let cache_path = format!("raw_data/noaa/{}_{}.json", state_id, data_type);
    let response_body = match load_data_from_file(&cache_path).await {
        Ok(body) => body,
        Err(_) => {
            let response_body =
                (|| async { data_request(token, state_id, data_type, &client).await })
                    .retry(ExponentialBackoff::default())
                    .await?;
            fs::write(&cache_path, &response_body).await?;
            response_body
        }
    };

    let values: Vec<f64> = serde_json::from_str::<GetResponse<Data>>(&response_body)?
        .results
        .into_iter()
        .map(|data| data.value)
        .collect();
    let value = values.iter().sum::<f64>() / values.len() as f64;

    Ok((state_id, value))
}

/// Get the values of a specific data type for all states. Returns code -> value map.
async fn get_data<'a>(
    token: &str,
    data_type: &str,
    states: &'a HashMap<ID, Code>,
    client: &Client,
) -> Result<HashMap<&'a Code, f64>> {
    Ok(try_join_all(
        states
            .iter()
            .map(|(id, _code)| get_data_for_state(token, id, data_type, client)),
    )
    .await?
    .into_iter()
    .filter_map(|(id, value)| Some((states.get(id)?, value)))
    .collect())
}

const DATA_TYPE_ANNUAL_TEMP: &str = "ANN-TAVG-NORMAL";
const DATA_TYPE_AUTUMN_TEMP: &str = "SON-TAVG-NORMAL";
const DATA_TYPE_SPRING_TEMP: &str = "MAM-TAVG-NORMAL";
const DATA_TYPE_SUMMER_TEMP: &str = "JJA-TAVG-NORMAL";
const DATA_TYPE_WINTER_TEMP: &str = "DJF-TAVG-NORMAL";

async fn get_annual_temp(token: &str, states: &HashMap<ID, Code>, client: &Client) -> Result<()> {
    let data = get_data(token, DATA_TYPE_ANNUAL_TEMP, states, client).await?;
    fs::write("generated/annual_temp.json", serde_json::to_string(&data)?).await?;
    Ok(())
}

async fn get_autumn_temp(token: &str, states: &HashMap<ID, Code>, client: &Client) -> Result<()> {
    let data = get_data(token, DATA_TYPE_AUTUMN_TEMP, states, client).await?;
    fs::write("generated/autumn_temp.json", serde_json::to_string(&data)?).await?;
    Ok(())
}

async fn get_spring_temp(token: &str, states: &HashMap<ID, Code>, client: &Client) -> Result<()> {
    let data = get_data(token, DATA_TYPE_SPRING_TEMP, states, client).await?;
    fs::write("generated/spring_temp.json", serde_json::to_string(&data)?).await?;
    Ok(())
}

async fn get_summer_temp(token: &str, states: &HashMap<ID, Code>, client: &Client) -> Result<()> {
    let data = get_data(token, DATA_TYPE_SUMMER_TEMP, states, client).await?;
    fs::write("generated/summer_temp.json", serde_json::to_string(&data)?).await?;
    Ok(())
}

async fn get_winter_temp(token: &str, states: &HashMap<ID, Code>, client: &Client) -> Result<()> {
    let data = get_data(token, DATA_TYPE_WINTER_TEMP, states, client).await?;
    fs::write("generated/winter_temp.json", serde_json::to_string(&data)?).await?;
    Ok(())
}

/// Get all the data for all the types for all the states and write to generated files
async fn get_all_data(token: &str, states: HashMap<ID, Code>, client: &Client) -> Result<()> {
    get_annual_temp(token, &states, client).await?;
    get_spring_temp(token, &states, client).await?;
    get_summer_temp(token, &states, client).await?;
    get_autumn_temp(token, &states, client).await?;
    get_winter_temp(token, &states, client).await?;
    Ok(())
}

/// Fetch weather data from NOAA. Intermediate results are stored in raw_data, final results in
/// generated. Requires a NOAA_TOKEN env var (can be in .env).
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let token = env::var("NOAA_TOKEN")?;
    let client = Client::new();
    let states = get_states(&token, &client).await?;
    get_all_data(&token, states, &client).await?;
    println!("Loaded NOAA data successfully");
    Ok(())
}
