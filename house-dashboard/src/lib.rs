// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main module

#![cfg_attr(not(doctest), doc = include_str!("../../Readme.md"))]
#![deny(
    missing_docs,
    clippy::cargo,
    clippy::pedantic,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
#![deny(
    clippy::allow_attributes_without_reason,
    clippy::clone_on_ref_ptr,
    clippy::else_if_without_else,
    clippy::expect_used,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::pattern_type_mismatch,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::str_to_string,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used,
    clippy::verbose_file_reads
)]

use std::ffi::OsStr;
use std::fs::{read, read_dir, read_to_string};
use std::io::{BufWriter, Cursor};
use std::path::Path;

use async_std::fs::write;

use tracing::{debug, trace};

use miette::{miette, IntoDiagnostic, Report, WrapErr};

use clap::Parser;

use toml::from_str as from_toml_str;

use plotters::style::{register_font, FontStyle};

use futures::{stream::FuturesUnordered, StreamExt};

use image::{ImageFormat, RgbImage};

use house_dashboard_common::configuration::StyleConfiguration;

mod commandline;
use self::commandline::Arguments;

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
    let arguments = Arguments::parse();
    setup_logging(arguments.verbosity)?;

    let (style_configuration, influxdb_configuration) =
        parse_configuration(&arguments.configuration_directory_path)
            .wrap_err("cannot parse configuration")?;

    trace!("Style configuration: {:?}", style_configuration);
    trace!("InfluxDB configuration: {:?}", influxdb_configuration);

    load_font(
        &style_configuration.font_name,
        &arguments
            .configuration_directory_path
            .join(&style_configuration.font_path),
    )?;

    let charts_configurations =
        parse_charts_configurations(&arguments.configuration_directory_path)
            .wrap_err("cannot parse charts configurations")?;

    trace!("Charts configurations: {:?}", charts_configurations);

    let mut tasks: FuturesUnordered<_> = charts_configurations
        .into_iter()
        .enumerate()
        .map(|(i, chart_configuration)| chart_configuration.process(&style_configuration, i))
        .collect();

    while let Some(result) = tasks.next().await {
        let (index, bytes) = result?;
        save_chart(index, bytes, style_configuration.resolution)
            .await
            .wrap_err("cannot save image")?;
    }

    Ok(())
}

/// Parse common configuration from configuration directory
fn parse_configuration(
    configuration_directory_path: &Path,
) -> Result<(StyleConfiguration, InfluxdbConfiguration), Report> {
    let style_configuration_path = configuration_directory_path.join("style.toml");
    let raw_style_configuration = read_to_string(&style_configuration_path)
        .into_diagnostic()
        .wrap_err("cannot read style configuration file")?;
    let style_configuration: StyleConfiguration = from_toml_str(&raw_style_configuration)
        .into_diagnostic()
        .wrap_err("cannot parse style configuration file")?;

    let influxdb_configuration_path = configuration_directory_path.join("influxdb.toml");
    let raw_influxdb_configuration = read_to_string(&influxdb_configuration_path)
        .into_diagnostic()
        .wrap_err("cannot read InfluxDB configuration file")?;
    let influxdb_configuration: InfluxdbConfiguration = from_toml_str(&raw_influxdb_configuration)
        .into_diagnostic()
        .wrap_err("cannot parse InfluxDB configuration file")?;

    Ok((style_configuration, influxdb_configuration))
}

/// Parse charts configuration from configuration directory
fn parse_charts_configurations(
    configuration_directory_path: &Path,
) -> Result<Vec<ChartConfiguration>, Report> {
    let entries: Vec<Result<Option<ChartConfiguration>, Report>> =
        read_dir(configuration_directory_path)
            .into_diagnostic()
            .wrap_err("Iterating over files in configuration directory")?
            .map(|result| {
                result.map(|dir_entry| {
                    parse_chart_configuration(&dir_entry.path())
                        .wrap_err(format!("Parsing file {}", dir_entry.path().display()))
                })
            })
            .collect::<Result<_, std::io::Error>>()
            .into_diagnostic()?;

    let entries: Result<Vec<Option<ChartConfiguration>>, Report> = entries.into_iter().collect();
    let entries: Vec<ChartConfiguration> = entries?.into_iter().flatten().collect();

    Ok(entries)
}

/// Parse individual chart configuration from file
fn parse_chart_configuration(path: &Path) -> Result<Option<ChartConfiguration>, Report> {
    if path
        .file_stem()
        .map_or(Some(""), OsStr::to_str)
        .unwrap_or("")
        .starts_with(|c: char| c.is_ascii_digit())
        && path.extension().map(OsStr::to_str) == Some(Some("toml"))
    {
        debug!("Processing path {}", path.display());
        let raw_configuration = read_to_string(path)
            .into_diagnostic()
            .wrap_err("Reading configuration file")?;
        let configuration: ChartConfiguration = from_toml_str(&raw_configuration)
            .into_diagnostic()
            .wrap_err("Parsing configuration file")?;
        Ok(Some(configuration))
    } else {
        Ok(None)
    }
}

/// Load custom font from a TTF or OTF file
fn load_font(name: &str, path: &Path) -> Result<(), Report> {
    let font_bytes = read(path).into_diagnostic()?.into_boxed_slice();
    let font_bytes: &'static [u8] = Box::leak(font_bytes);
    register_font(name, FontStyle::Normal, font_bytes).map_err(|_| miette!("Cannot load font"))?;
    Ok(())
}

/// Save a chart to a file
async fn save_chart(
    index: usize,
    bytes: Vec<u8>,
    (width, height): (u32, u32),
) -> Result<(), Report> {
    let filename = format!("{:02}.bmp", index + 1);
    let path = Path::new(&filename);

    let image =
        RgbImage::from_raw(width, height, bytes).ok_or_else(|| miette!("invalid image data"))?;

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image
        .write_to(&mut buffer, ImageFormat::Bmp)
        .into_diagnostic()?;

    let buffer = buffer.into_inner().into_diagnostic()?.into_inner();
    write(path, &buffer).await.into_diagnostic()?;
    Ok(())
}
