[EIA](eia/README.md) is the U.S. Energy Information Administration which publishes some JSON data
 about energy consumption/production.

## How to Use
Collect this data with `cargo run --bin eia` from the root dir. Requires an api key 
as an environment variable called `EIA_KEY` which can be acquired from https://www.eia.gov/developer/.

## Sources Generated
- co2_emissions.json
- percent_renewable.json
 