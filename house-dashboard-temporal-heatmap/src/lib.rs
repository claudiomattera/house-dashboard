// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating temporal heatmap charts

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
#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use tracing::{debug, instrument};

use miette::miette;
use miette::{IntoDiagnostic, Report, WrapErr};

use chrono::{DateTime, Utc};

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_temporal_heatmap;

mod configuration;
pub use self::configuration::TemporalHeatMapConfiguration;

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for temporal heatmap
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "temporal_heatmap",
    skip(influxdb_client, temporal_heatmap_configuration, style_configuration)
)]
pub async fn process_temporal_heatmap(
    influxdb_client: &InfluxDBClient,
    temporal_heatmap_configuration: &TemporalHeatMapConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let time_series = fetch_data(influxdb_client, temporal_heatmap_configuration)
        .await
        .wrap_err("cannot fetch data for temporal heatmap")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_temporal_heatmap(
        temporal_heatmap_configuration,
        &time_series,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw temporal heatmap")?;

    Ok(buffer)
}

/// Fetch data for temporal heatmap
///
/// # Errors
///
/// Return and error when data could not be fetched
async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    temporal_heatmap_configuration: &TemporalHeatMapConfiguration,
) -> Result<Vec<(DateTime<Utc>, f64)>, Report> {
    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND {tag} = '{tag_value}'
        GROUP BY time({period}),{tag} FILL(previous)",
        scale = temporal_heatmap_configuration.scale.unwrap_or(1.0),
        aggregator = temporal_heatmap_configuration
            .aggregator
            .clone()
            .unwrap_or_else(|| "mean".to_owned()),
        field = temporal_heatmap_configuration.field,
        database = temporal_heatmap_configuration.database,
        measurement = temporal_heatmap_configuration.measurement,
        tag = temporal_heatmap_configuration.tag,
        tag_value = &temporal_heatmap_configuration.tag_value,
        period = temporal_heatmap_configuration.period.to_query_group(),
        how_long_ago = temporal_heatmap_configuration.period.how_long_ago(),
    );

    debug!("Query: {}", query);

    let mut time_seriess = match influxdb_client
        .fetch_tagged_dataframes(&query, &temporal_heatmap_configuration.tag)
        .await
    {
        Ok(time_seriess) => Ok(time_seriess),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch time-series")?;

    let time_series = time_seriess
        .remove(&temporal_heatmap_configuration.tag_value)
        .ok_or(miette!(
            "Missing data for {} = '{}'",
            temporal_heatmap_configuration.tag,
            temporal_heatmap_configuration.tag_value
        ))?;

    let time_series = time_series
        .into_iter()
        .filter(|&(ref _instant, ref value)| !value.is_nan())
        .collect();

    Ok(time_series)
}
