# whatstatedata

A data parser / compiler for whatstateshouldilivein.com.

Each data source has its own binary crate to load/parse it. They're all different
depending on where the data was found and what format it was available in.

## How to Use

1. The [sources.json](sources.json) file is edited by hand. It's the entrypoint from
   the UI to all the actual data. It contains meta stuff like units and description.
2. Many of the crates require environment variables to run. When in use, [dotenv](https://crates.io/crates/dotenv)
   will also be used so environment variables can be put in a `.env` file.
3. Some sources may require manually downloading data for consumption. That data
   as well as any intermediate cached data will be stored in `raw_data`.
4. All output data will be put in the `generated` folder. This stuff, along with sources
   should be copied to the front end project (or whatever else might end up using this).

## Sources

[EIA](eia/README.md) is the U.S. Energy Information Administration which publishes some JSON data
about energy consumption/production.

[Maryland](maryland/README.md) is several great JSON data sources open to the public
collected by the state of Maryland.

[NOAA](noaa/README.md) is the National Oceanic and Atmospheric Administration with some good weather info.

[NPS](nps/README.md) is the National Park Service.

[solar_prices](solar_prices/README.md) data comes from a project called "Tracking the Sun"
run by Berkeley Lab.
