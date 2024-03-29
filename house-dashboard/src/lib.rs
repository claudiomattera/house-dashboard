// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main module

#![cfg_attr(not(doctest), doc = include_str!("../../Readme.md"))]

use std::ffi::OsStr;
use std::io::{BufWriter, Cursor};
use std::path::Path;

use async_std::fs::read as read_file;
use async_std::fs::read_dir;
use async_std::fs::read_to_string as read_file_to_string;
use async_std::fs::write as write_file;

use tracing::{debug, info, trace, warn};

use miette::{miette, IntoDiagnostic, Report, WrapErr};

use toml::from_str as from_toml_str;

use plotters::style::{register_font, FontStyle};

use futures::stream::iter as future_from_iter;
use futures::{future::ready, stream::FuturesUnordered, StreamExt};

use image::{ImageFormat, RgbImage};

#[cfg(target_arch = "x86_64")]
use extrasafe::{
    builtins::{danger_zone::ForkAndExec, BasicCapabilities, Networking, SystemIO},
    SafetyContext,
};

use isahc::{
    auth::{Authentication, Credentials},
    config::{CaCertificate, Configurable, SslOption},
    HttpClient,
};

use house_dashboard_common::configuration::StyleConfiguration;

use house_dashboard_influxdb::InfluxDBClient;

mod commandline;
use self::commandline::parse_command_line;

mod configuration;
use self::configuration::Chart as ChartConfiguration;
use self::configuration::Influxdb as InfluxdbConfiguration;

mod logging;
use self::logging::setup as setup_logging;

/// Main function
///
/// # Errors
///
/// Return an error when anything fails
pub async fn main() -> Result<(), Report> {
    let arguments = parse_command_line();
    setup_logging(arguments.verbosity.try_into().into_diagnostic()?)?;

    #[cfg(target_arch = "x86_64")]
    setup_allowed_syscalls()?;

    let (style_configuration, influxdb_configuration) =
        parse_configuration(&arguments.configuration_directory_path)
            .await
            .wrap_err("cannot parse configuration")?;

    trace!("Style configuration: {:?}", style_configuration);
    trace!("InfluxDB configuration: {:?}", influxdb_configuration);

    load_font(
        &style_configuration.font_name,
        &arguments
            .configuration_directory_path
            .join(&style_configuration.font_path),
    )
    .await?;

    let charts_configurations =
        parse_charts_configurations(&arguments.configuration_directory_path)
            .await
            .wrap_err("cannot parse charts configurations")?;

    trace!("Charts configurations: {:?}", charts_configurations);

    let influxdb_client = create_influxdb_client(influxdb_configuration)?;

    let mut tasks: FuturesUnordered<_> = charts_configurations
        .into_iter()
        .enumerate()
        .map(|(i, chart_configuration)| {
            chart_configuration.process(influxdb_client.clone(), &style_configuration, i)
        })
        .collect();

    while let Some(result) = tasks.next().await {
        let (index, bytes) = result?;
        save_chart(
            index,
            bytes,
            style_configuration.resolution,
            &arguments.output_directory_path,
        )
        .await
        .wrap_err("cannot save image")?;
    }

    Ok(())
}

/// Setup allowed syscalls
#[cfg(target_arch = "x86_64")]
fn setup_allowed_syscalls() -> Result<(), Report> {
    let safety_context = SafetyContext::new();

    let safety_context = safety_context
        .enable(BasicCapabilities)
        .into_diagnostic()
        .wrap_err("cannot enable basic syscalls")?;

    let safety_context = safety_context
        .enable(
            SystemIO::nothing()
                .allow_read()
                .allow_write()
                .allow_ioctl()
                .allow_metadata()
                .allow_open()
                .yes_really()
                .allow_close(),
        )
        .into_diagnostic()
        .wrap_err("cannot enable system IO syscalls")?;

    let safety_context = safety_context
        .enable(Networking::nothing().allow_start_tcp_clients())
        .into_diagnostic()
        .wrap_err("cannot enable networking syscalls")?;

    let safety_context = safety_context
        .enable(ForkAndExec)
        .into_diagnostic()
        .wrap_err("cannot enable fork and exec syscalls")?;

    safety_context
        .apply_to_all_threads()
        .into_diagnostic()
        .wrap_err("cannot apply safety context")?;

    Ok(())
}

/// Parse common configuration from configuration directory
async fn parse_configuration(
    configuration_directory_path: &Path,
) -> Result<(StyleConfiguration, InfluxdbConfiguration), Report> {
    let style_configuration_path = configuration_directory_path.join("style.toml");
    let raw_style_configuration = read_file_to_string(style_configuration_path)
        .await
        .into_diagnostic()
        .wrap_err("cannot read style configuration file")?;
    let style_configuration: StyleConfiguration = from_toml_str(&raw_style_configuration)
        .into_diagnostic()
        .wrap_err("cannot parse style configuration file")?;

    let influxdb_configuration_path = configuration_directory_path.join("influxdb.toml");
    let raw_influxdb_configuration = read_file_to_string(influxdb_configuration_path)
        .await
        .into_diagnostic()
        .wrap_err("cannot read InfluxDB configuration file")?;
    let influxdb_configuration: InfluxdbConfiguration = from_toml_str(&raw_influxdb_configuration)
        .into_diagnostic()
        .wrap_err("cannot parse InfluxDB configuration file")?;

    Ok((style_configuration, influxdb_configuration))
}

/// Parse charts configuration from configuration directory
async fn parse_charts_configurations(
    configuration_directory_path: &Path,
) -> Result<Vec<ChartConfiguration>, Report> {
    let results: Vec<Result<Option<ChartConfiguration>, Report>> =
        read_dir(configuration_directory_path)
            .await
            .into_diagnostic()
            .wrap_err("cannot iterate over files in configuration directory")?
            .map(|result| result.map(|dir_entry| dir_entry.path()))
            .flat_map(future_from_iter)
            .filter(|path| ready(path.extension() == Some(OsStr::new("toml"))))
            .filter(|path| ready(path.file_name() != Some(OsStr::new("influxdb.toml"))))
            .filter(|path| ready(path.file_name() != Some(OsStr::new("style.toml"))))
            .then(|path| async move {
                parse_chart_configuration(path.as_ref())
                    .await
                    .wrap_err(format!("cannot parse file {}", path.display()))
            })
            .collect::<Vec<Result<Option<ChartConfiguration>, Report>>>()
            .await;

    let result: Vec<Option<ChartConfiguration>> = results
        .into_iter()
        .collect::<Result<Vec<Option<ChartConfiguration>>, Report>>()?;

    let entries: Vec<ChartConfiguration> = result
        .into_iter()
        .flatten()
        .collect::<Vec<ChartConfiguration>>();

    Ok(entries)
}

/// Parse individual chart configuration from file
async fn parse_chart_configuration(path: &Path) -> Result<Option<ChartConfiguration>, Report> {
    if path
        .file_stem()
        .map_or(Some(""), OsStr::to_str)
        .unwrap_or("")
        .starts_with(|c: char| c.is_ascii_digit())
        && path.extension().map(OsStr::to_str) == Some(Some("toml"))
    {
        debug!("Processing path {}", path.display());
        let raw_configuration = read_file_to_string(path)
            .await
            .into_diagnostic()
            .wrap_err("cannot read chart configuration file")?;
        let configuration: ChartConfiguration = from_toml_str(&raw_configuration)
            .into_diagnostic()
            .wrap_err("cannot parse chart configuration file")?;
        Ok(Some(configuration))
    } else {
        Ok(None)
    }
}

/// Load custom font from a TTF or OTF file
async fn load_font(name: &str, path: &Path) -> Result<(), Report> {
    let font_bytes = read_file(path)
        .await
        .into_diagnostic()
        .wrap_err("cannot read font file")?
        .into_boxed_slice();
    let font_bytes: &'static [u8] = Box::leak(font_bytes);
    register_font(name, FontStyle::Normal, font_bytes).map_err(|_| miette!("Cannot load font"))?;
    Ok(())
}

/// Create an InfluxDB client
fn create_influxdb_client(
    influxdb_configuration: InfluxdbConfiguration,
) -> Result<InfluxDBClient, Report> {
    let mut http_client_builder = HttpClient::builder()
        .authentication(Authentication::basic())
        .credentials(Credentials::new(
            influxdb_configuration.username,
            influxdb_configuration.password,
        ));

    if let Some(path) = influxdb_configuration.cacert {
        info!("Adding custom CA certificate {}", path.display());
        http_client_builder = http_client_builder.ssl_ca_certificate(CaCertificate::file(path));
    }

    if influxdb_configuration
        .dangerously_accept_invalid_certs
        .unwrap_or(false)
    {
        warn!("Accepting invalid TLS certificates");
        http_client_builder =
            http_client_builder.ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS);
    }

    let http_client = http_client_builder
        .build()
        .into_diagnostic()
        .wrap_err("Creating HTTP client")?;

    let influxdb_client = InfluxDBClient::new(influxdb_configuration.url, http_client);

    Ok(influxdb_client)
}

/// Save a chart to a file
async fn save_chart(
    index: usize,
    bytes: Vec<u8>,
    (width, height): (u32, u32),
    output_directory_path: &Path,
) -> Result<(), Report> {
    let filename = format!("{:02}.bmp", index + 1);
    let path = output_directory_path.join(&filename);

    let image =
        RgbImage::from_raw(width, height, bytes).ok_or_else(|| miette!("invalid image data"))?;

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image
        .write_to(&mut buffer, ImageFormat::Bmp)
        .into_diagnostic()?;

    let buffer = buffer.into_inner().into_diagnostic()?.into_inner();
    write_file(path, &buffer).await.into_diagnostic()?;
    Ok(())
}
