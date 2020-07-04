// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::env;
use std::path::Path;
use std::process::exit;
use std::fs::remove_file;

use anyhow::{Context, Result};

use clap::{Arg, ArgMatches, SubCommand};
use clap::{app_from_crate, crate_name, crate_version, crate_authors, crate_description};

use glob::glob;

mod backend;
mod chart;
mod configuration;
mod error;
mod framebuffer;
mod influxdb;
mod types;

use crate::backend::OtherBackendType;
use crate::configuration::{Configuration, ChartConfiguration, TrendConfiguration};
use crate::influxdb::InfluxdbClient;

fn main() {
    exit(
        match inner_main() {
            Ok(()) => 0,
            Err(error) => {
                error!("Error: {:?}", error);
                1
            },
        }
    );
}

fn inner_main() -> Result<()> {
    let matches = parse_arguments();
    setup_logging(matches.occurrences_of("verbosity"));

    if matches.subcommand().1.is_none() {
        println!("{}", matches.usage());
        return Ok(());
    }

    debug!("Parsing configuration");
    let configuration_path = matches.value_of("configuration-path")
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
        ("display", Some(subcommand)) => {
            let device = subcommand.value_of("device")
                .map(Path::new)
                .expect("Missing argument \"device\"");
            info!("Displaying chart on framebuffer {}", device.display());

            let mut buffer = vec![0u8; (3 * configuration.style.resolution.0 * configuration.style.resolution.1) as usize];

            let time_seriess = influxdb_client.fetch_timeseries_by_tag(
                "SELECT mean(\"temperature\") AS \"mean_value\" FROM \"longterm\".\"autogen\".\"indoor_environment\" WHERE time > now() - 1d GROUP BY time(30m),room FILL(none)",
                "room",
            ).context("Failed to fetch time-series")?;

            chart::draw_chart(time_seriess, "Temperature", &Some("Temperature [C]".to_string()), 50, "%H:%M", OtherBackendType::new_from_frame_buffer(device, &mut buffer, (configuration.style.resolution.0, configuration.style.resolution.1)))
                .context("Failed to draw chart to framebuffer")?;
        }
        ("save", Some(subcommand)) => {
            debug!("Creating directory path");
            let directory_path = subcommand.value_of("path")
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
                let backend = OtherBackendType::new_from_path(&chart_path, configuration.style.resolution);

                match chart {
                    ChartConfiguration::Trend(chart) => generate_trend_chart(chart, &influxdb_client, backend),
                }.context("Failed to save chart to file")?
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
            SubCommand::with_name("display")
                .about("Display charts on the framebuffer")
                .arg(
                    Arg::with_name("device")
                        .short("d")
                        .long("device")
                        .required(true)
                        .help("Path to framebuffer device")
                        .takes_value(true),
                )
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
                )
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
            backend: OtherBackendType,
        ) -> Result<()> {
    debug!("Generating trend chart");

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &chart.query,
        &chart.tag,
    ).context("Failed to fetch data from database")?;
    chart::draw_chart(
        time_seriess,
        &chart.title,
        &chart.ylabel,
        50,
        &chart.xlabel_format,
        backend,
    ).context("Failed to draw chart")?;
    Ok(())
}
