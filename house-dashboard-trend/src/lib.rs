// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating trend charts

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
use std::fmt::Error as FmtError;
use std::fmt::Write;

use time::Duration;

use tracing::{debug, instrument};

use miette::{IntoDiagnostic, Report, WrapErr};

use chrono::{DateTime, Utc};

use plotters::backend::BitMapBackend;

use house_dashboard_common::configuration::StyleConfiguration;

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

mod chart;
pub use self::chart::draw_trend;

mod configuration;
pub use self::configuration::TrendConfiguration;

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for trend
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "trend",
    skip(influxdb_client, trend_configuration, style_configuration)
)]
pub async fn process_trend(
    influxdb_client: &InfluxDBClient,
    trend_configuration: &TrendConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let time_seriess = fetch_data(influxdb_client, trend_configuration)
        .await
        .wrap_err("cannot fetch data for trend")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_trend(
        trend_configuration,
        &time_seriess,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw trend")?;

    Ok(buffer)
}

/// Fetch data for trend
///
/// # Errors
///
/// Return and error when data could not be fetched
async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    trend_configuration: &TrendConfiguration,
) -> Result<HashMap<String, Vec<(DateTime<Utc>, f64)>>, Report> {
    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago}
        GROUP BY time({period}),{tag} FILL(none)",
        database = "telegraf",
        scale = trend_configuration.scale.unwrap_or(1.0),
        aggregator = trend_configuration
            .aggregator
            .clone()
            .unwrap_or_else(|| "mean".to_owned()),
        field = trend_configuration.field,
        measurement = trend_configuration.measurement,
        tag = trend_configuration.tag,
        period = trend_configuration
            .how_often
            .as_ref()
            .map_or_else(|| Ok("1h".to_owned()), |d| duration_to_query(&d.duration),)
            .into_diagnostic()?,
        how_long_ago =
            duration_to_query(&trend_configuration.how_long_ago.duration).into_diagnostic()?,
    );

    debug!("Query: {}", query);

    let time_seriess = match influxdb_client
        .fetch_tagged_dataframes(&query, &trend_configuration.tag)
        .await
    {
        Ok(time_seriess) => Ok(time_seriess),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch time-series")?;

    Ok(time_seriess)
}

/// Convert a duration to a duration string
fn duration_to_query(duration: &Duration) -> Result<String, FmtError> {
    let mut string = String::new();

    let seconds = duration.whole_seconds();
    if seconds > 0 {
        write!(&mut string, "{seconds}s")?;
    }

    Ok(string)
}
