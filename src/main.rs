// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::collections::HashMap;
use std::env;
use std::fs::remove_file;
use std::path::Path;
use std::process::exit;

use anyhow::{Context, Result};

use clap::{app_from_crate, crate_name, crate_version, crate_authors, crate_description};
use clap::{Arg, ArgMatches, SubCommand};

use glob::glob;

use plotters::drawing::BitMapBackend;

mod chart;
mod colormap;
mod configuration;
mod error;
mod framebuffer;
mod influxdb;
mod palette;
mod types;

use crate::configuration::{
    Configuration, ChartConfiguration, GeographicalHeatMapConfiguration,
    GeographicalRegionConfiguration, TemporalHeatMapConfiguration,
    TrendConfiguration, StyleConfiguration
};
use crate::influxdb::InfluxdbClient;
use crate::error::DashboardError;

fn main() {
    exit(match inner_main() {
        Ok(()) => 0,
        Err(error) => {
            error!("Error: {:?}", error);
            1
        },
    });
}

fn inner_main() -> Result<()> {
    let matches = parse_arguments();
    setup_logging(matches.occurrences_of("verbosity"));

    if matches.subcommand().1.is_none() {
        println!("{}", matches.usage());
        return Ok(());
    }

    debug!("Parsing configuration");
    let configuration_path = matches
        .value_of("configuration-path")
        .map(Path::new)
        .expect("Missing argument \"configuration\"");
    let configuration = parse_configuration(configuration_path)
        .context("Could not load configuration")?;

    debug!("Creating InfluxDB client");
    let influxdb_client = InfluxdbClient::new(
        configuration.influxdb.url,
        configuration.influxdb.database,
        configuration.influxdb.username,
        configuration.influxdb.password,
        configuration.influxdb.cacert,
    );

    debug!("Matching subcommand");
    match matches.subcommand() {
        ("save", Some(subcommand)) => {
            debug!("Creating directory path");
            let directory_path = subcommand
                .value_of("path")
                .map(Path::new)
                .expect("Missing argument \"path\"");
            info!("Saving chart to directory {}", directory_path.display());
            if subcommand.is_present("clear") {
                info!("Removing existing BMP files");
                for image_path_result in glob(directory_path.join("*.bmp").as_path().to_str().expect("Invalid path"))? {
                    let image_path =  image_path_result?;
                    debug!("Removing {}", image_path.display());
                    remove_file(image_path)?;
                }
            }

            let n = configuration.charts.len();
            for (i, chart) in (1..).zip(configuration.charts) {
                info!("Generating chart {} of {}", i, n);

                let chart_path = directory_path.join(format!("{}.bmp", i));
                let backend = BitMapBackend::new(&chart_path, configuration.style.resolution);

                let result = match chart {
                    ChartConfiguration::Trend(chart) => {
                        generate_trend_chart(chart, &influxdb_client, &configuration.style, backend)
                    }
                    ChartConfiguration::GeographicalMap(chart) => {
                        let regions = configuration.regions.clone().unwrap_or_else(Vec::new);
                        generate_geographical_map_chart(chart, regions, &influxdb_client, &configuration.style, backend)
                    }
                    ChartConfiguration::TemporalHeatMap(chart) => {
                        generate_temporal_heat_map_chart(chart, &influxdb_client, &configuration.style, backend)
                    }
                }.context("Failed to save chart to file");

                if let Err(error) = result {
                    error!("Error: {:?}", error);
                }
            }
        }
        _ => println!("{}", matches.usage()),
    }

    Ok(())
}

fn parse_arguments() -> ArgMatches<'static> {
    app_from_crate!()
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity")
        )
        .arg(
            Arg::with_name("configuration-path")
                .short("c")
                .long("configuration")
                .required(true)
                .help("Path to configuration file")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("save")
                .about("Save charts to files")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .required(true)
                        .help("Path to charts directory")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("clear")
                        .long("clear")
                        .help("Clears all .bmp files in charts directory"),
                ),
        )
        .get_matches()
}

fn setup_logging(verbosity: u64) {
    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "info,house_dashboard=debug",
        _ => "debug",
    };
    let filter = env_logger::Env::default().default_filter_or(default_log_filter);
    env_logger::Builder::from_env(filter).format_timestamp(None).init();
}

fn parse_configuration(
            configuration_path: &Path
        ) -> Result<Configuration> {
    let mut file = std::fs::File::open(configuration_path)?;
    let mut contents = String::new();
    use std::io::Read;
    file.read_to_string(&mut contents)?;
    let configuration: Configuration = toml::from_str(&contents)?;
    Ok(configuration)
}

fn generate_trend_chart(
            chart: TrendConfiguration,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            backend: BitMapBackend,
        ) -> Result<()> {
    debug!("Generating trend chart");

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &chart.query,
        &chart.tag,
    )
    .context("Failed to fetch data from database")?;

    chart::draw_trend_chart(
        time_seriess,
        &chart.title,
        &chart.ylabel,
        50,
        &chart.xlabel_format,
        chart.tag_values,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    Ok(())
}

fn generate_geographical_map_chart(
            chart: GeographicalHeatMapConfiguration,
            regions_configurations: Vec<GeographicalRegionConfiguration>,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            backend: BitMapBackend,
        ) -> Result<()> {
    debug!("Generating geographical map chart");

    let mut regions = HashMap::<String, Vec<(f64, f64)>>::new();
    for region in regions_configurations {
        regions.insert(region.name, region.coordinates);
    }

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &chart.query,
        &chart.tag,
    )
    .context("Failed to fetch data from database")?;

    let values: HashMap<String, Option<f64>> = time_seriess.iter()
        .map(|(region, time_series)| (region.to_owned(), time_series.first().map(|o| o.1)))
        .collect();

    chart::draw_geographical_heat_map_chart(
        values,
        chart.bounds,
        chart.colormap,
        &chart.title,
        &chart.unit,
        regions,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    Ok(())
}

fn generate_temporal_heat_map_chart(
            chart: TemporalHeatMapConfiguration,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            backend: BitMapBackend,
        ) -> Result<()> {
    debug!("Generating temporal heat map chart");

    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND {tag} = '{tag_value}'
        GROUP BY time({period}),{tag} FILL(none)",
        scale = chart.scale.unwrap_or(1.0),
        aggregator = chart.aggregator.unwrap_or_else(|| "mean".to_owned()),
        field = chart.field,
        database = chart.database,
        measurement = chart.measurement,
        tag = chart.tag,
        tag_value = chart.tag_value,
        period = chart.period.to_query_group(),
        how_long_ago = chart.period.how_long_ago(),
    );

    debug!("Query: {}", query);

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &query,
        &chart.tag,
    )
    .context("Failed to fetch data from database")?;

    let time_series = time_seriess
        .get(&chart.tag_value)
        .ok_or(DashboardError::NonexistingTagValue(chart.tag_value))?;

    chart::draw_temporal_heat_map_chart(
        time_series.to_owned(),
        chart.period,
        &chart.title,
        &chart.unit,
        chart.bounds,
        chart.colormap,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    Ok(())
}
