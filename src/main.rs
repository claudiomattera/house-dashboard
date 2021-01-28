// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use tracing::*;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_log::LogTracer;

use std::collections::HashMap;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{Context, Result};

use chrono::Duration;

use clap::{app_from_crate, crate_name, crate_version, crate_authors, crate_description};
use clap::{Arg, ArgMatches, SubCommand};

use glob::glob;

use plotters::drawing::BitMapBackend;

use futures::future::try_join_all;

use indicatif::ProgressBar;

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
    GeographicalRegionConfiguration, ImageConfiguration,
    InfrastructureSummaryConfiguration, TemporalHeatMapConfiguration,
    TrendConfiguration, StyleConfiguration
};
use crate::influxdb::InfluxdbClient;
use crate::error::DashboardError;

#[tokio::main]
async fn main() {
    exit(match inner_main().await {
        Ok(()) => 0,
        Err(error) => {
            error!("Error: {:?}", error);
            1
        },
    });
}

async fn inner_main() -> Result<()> {
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
        configuration.influxdb.dangerously_accept_invalid_certs.unwrap_or(false),
    )?;

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

            let bar = if false {
                ProgressBar::new(configuration.charts.len() as u64)
            } else {
                ProgressBar::hidden()
            };

            type Out = Result<(), anyhow::Error>;
            let mut tasks: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = Out>>>> = Vec::new();

            info!("Generating {} charts...", configuration.charts.len());

            for (i, chart) in (1..).zip(configuration.charts) {
                let chart_path = directory_path
                    .join(format!("{:02}.bmp", i))
                    .to_owned();

                match chart {
                    ChartConfiguration::Trend(chart) => {
                        let task = generate_trend_chart(chart, &influxdb_client, &configuration.style, chart_path, configuration.style.resolution, &bar);
                        tasks.push(Box::pin(task));
                    }
                    ChartConfiguration::GeographicalHeatMap(chart) => {
                        let regions = configuration.regions.clone().unwrap_or_else(Vec::new);
                        let task = generate_geographical_map_chart(chart, regions, &influxdb_client, &configuration.style, chart_path, configuration.style.resolution, &bar);
                        tasks.push(Box::pin(task));
                    }
                    ChartConfiguration::TemporalHeatMap(chart) => {
                        let task = generate_temporal_heat_map_chart(chart, &influxdb_client, &configuration.style, chart_path, configuration.style.resolution, &bar);
                        tasks.push(Box::pin(task));
                    }
                    ChartConfiguration::Image(image_configuration) => {
                        let task = generate_image(image_configuration, chart_path, configuration.style.resolution, &bar);
                        tasks.push(Box::pin(task));
                    }
                    ChartConfiguration::InfrastructureSummary(infrastructure_summary_configuration) => {
                        let task = generate_infrastructure_summary(infrastructure_summary_configuration, &influxdb_client, &configuration.style, chart_path, configuration.style.resolution, &bar);
                        tasks.push(Box::pin(task));
                    }
                }
            };

            let _results: Vec<()> = try_join_all(tasks).await?;
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
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "info,house_dashboard=debug",
        _ => "debug",
    };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_log_filter));

    let formatting_layer = subscriber_fmt::layer()
        .with_target(false)
        .without_time();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");
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

#[tracing::instrument(
    name = "Generating a trend chart",
    skip(chart, influxdb_client, style, resolution, progress_bar),
    fields(
        path = %path.display(),
    )
)]
async fn generate_trend_chart(
            chart: TrendConfiguration,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            path: PathBuf,
            resolution: (u32, u32),
            progress_bar: &ProgressBar,
        ) -> Result<()> {
    let backend = BitMapBackend::new(&path, resolution);

    debug!("Generating trend chart");

    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago}
        GROUP BY time({period}),{tag} FILL(none)",
        scale = chart.scale.unwrap_or(1.0),
        aggregator = chart.aggregator.unwrap_or_else(|| "mean".to_owned()),
        field = chart.field,
        database = chart.database,
        measurement = chart.measurement,
        tag = chart.tag,
        period = chart.how_often
            .map(|d| duration_to_query(&d.duration))
            .unwrap_or_else(|| "1h".to_owned()),
        how_long_ago = duration_to_query(&chart.how_long_ago.duration),
    );

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &query,
        &chart.tag,
    )
    .await
    .context("Failed to fetch data from database")?;

    chart::draw_trend_chart(
        time_seriess,
        &chart.title,
        &chart.ylabel,
        &chart.yunit,
        50,
        &chart.xlabel_format,
        chart.precision.unwrap_or(0),
        chart.draw_last_value.unwrap_or(false),
        chart.hide_legend.unwrap_or(false),
        chart.tag_values,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    progress_bar.inc(1);

    Ok(())
}

#[tracing::instrument(
    name = "Generating a geographical map chart",
    skip(chart, regions_configurations, influxdb_client, style, resolution, progress_bar),
    fields(
        path = %path.display(),
    )
)]
async fn generate_geographical_map_chart(
            chart: GeographicalHeatMapConfiguration,
            regions_configurations: Vec<GeographicalRegionConfiguration>,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            path: PathBuf,
            resolution: (u32, u32),
            progress_bar: &ProgressBar,
        ) -> Result<()> {
    let backend = BitMapBackend::new(&path, resolution);

    debug!("Generating geographical map chart");

    let mut regions = HashMap::<String, Vec<(f64, f64)>>::new();
    for region in regions_configurations {
        regions.insert(region.name, region.coordinates);
    }

    let query = format!(
        "SELECT {scale} * last({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago}
        GROUP BY {tag} FILL(none)",
        scale = chart.scale.unwrap_or(1.0),
        field = chart.field,
        database = chart.database,
        measurement = chart.measurement,
        tag = chart.tag,
        how_long_ago = duration_to_query(&chart.how_long_ago.duration),
    );

    let time_seriess = influxdb_client.fetch_timeseries_by_tag(
        &query,
        &chart.tag,
    )
    .await
    .context("Failed to fetch data from database")?;

    let values: HashMap<String, Option<f64>> = time_seriess.iter()
        .map(|(region, time_series)| (region.to_owned(), time_series.first().map(|o| o.1)))
        .collect();

    chart::draw_geographical_heat_map_chart(
        values,
        chart.bounds,
        chart.precision.unwrap_or(0),
        chart.colormap,
        chart.reversed,
        &chart.title,
        &chart.unit,
        regions,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    progress_bar.inc(1);

    Ok(())
}

#[tracing::instrument(
    name = "Generating a temporal heatmap chart",
    skip(chart, influxdb_client, style, resolution, progress_bar),
    fields(
        path = %path.display(),
    )
)]
async fn generate_temporal_heat_map_chart(
            chart: TemporalHeatMapConfiguration,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            path: PathBuf,
            resolution: (u32, u32),
            progress_bar: &ProgressBar,
        ) -> Result<()> {
    let backend = BitMapBackend::new(&path, resolution);

    debug!("Generating temporal heat map chart");

    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND {tag} = '{tag_value}'
        GROUP BY time({period}),{tag} FILL(previous)",
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
    .await
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
        chart.precision.unwrap_or(0),
        chart.colormap,
        style,
        backend,
    )
    .context("Failed to draw chart")?;

    progress_bar.inc(1);

    Ok(())
}

#[tracing::instrument(
    name = "Generating an image chart",
    skip(image_configuration, resolution, progress_bar),
    fields(
        path = %path.display(),
    )
)]
async fn generate_image(
            image_configuration: ImageConfiguration,
            path: PathBuf,
            resolution: (u32, u32),
            progress_bar: &ProgressBar,
        ) -> Result<()> {
    let backend = BitMapBackend::new(&path, resolution);

    chart::draw_image(
        image_configuration.path,
        backend,
    )
    .context("Failed to draw image")?;

    progress_bar.inc(1);

    Ok(())
}

#[tracing::instrument(
    name = "Generating an infrastructure summary chart",
    skip(infrastructure_summary, influxdb_client, style, resolution, progress_bar),
    fields(
        path = %path.display(),
    )
)]
async fn generate_infrastructure_summary(
            infrastructure_summary: InfrastructureSummaryConfiguration,
            influxdb_client: &InfluxdbClient,
            style: &StyleConfiguration,
            path: PathBuf,
            resolution: (u32, u32),
            progress_bar: &ProgressBar,
        ) -> Result<()> {
    let backend = BitMapBackend::new(&path, resolution);

    let load_field = "load15";
    let n_cpus_field = "n_cpus";
    let database = "telegraf";
    let measurement = "system";
    let tag = "host";
    let filter_tag_name = "always-on";
    let filter_tag_value = "true";

    let hosts = influxdb_client.fetch_tag_values(
        database,
        measurement,
        tag,
        filter_tag_name,
        filter_tag_value,
    )
    .await
    .context("Failed to fetch data from database")?;

    debug!("Found {} hosts: {}", hosts.len(), hosts.iter().cloned().collect::<Vec<String>>().join(", "));

    let query = format!(
        "SELECT last({load_field}) / last({n_cpus_field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND \"{filter_tag_name}\" = '{filter_tag_value}'
        GROUP BY {tag}",
        load_field = load_field,
        n_cpus_field = n_cpus_field,
        database = database,
        measurement = measurement,
        tag = tag,
        filter_tag_name = filter_tag_name,
        filter_tag_value = filter_tag_value,
        how_long_ago = duration_to_query(&infrastructure_summary.how_long_ago.duration),
    );

    let loads = influxdb_client.fetch_timeseries_by_tag(
        &query,
        &tag,
    )
    .await
    .context("Failed to fetch data from database")?;

    chart::draw_infrastructure_summary(
        infrastructure_summary,
        hosts,
        loads,
        style,
        backend,
    )
    .context("Failed to draw image")?;

    progress_bar.inc(1);

    Ok(())
}

fn duration_to_query(duration: &Duration) -> String {
    let mut string = String::new();

    let seconds = duration.num_seconds();
    if seconds > 0 {
        string.push_str(&format!("{}s", seconds));
    }

    string
}
