use std::str::FromStr;

use anyhow::{anyhow, Result};

pub use demographic::demographic;
pub use quality_of_life::quality_of_life;
pub use taxes::taxes;
pub use workforce::workforce;

mod demographic;
mod quality_of_life;
mod taxes;
mod workforce;

pub static BASE_URL: &str = "https://opendata.maryland.gov/resource/";

pub fn parse_percent(val: &str) -> Result<f64> {
    let no_percent = if val.ends_with('%') {
        &val[..val.len() - 1]
    } else {
        val
    };
    f64::from_str(no_percent)
        .map(|f| f / 100.0)
        .map_err(|e| anyhow!(e))
}
