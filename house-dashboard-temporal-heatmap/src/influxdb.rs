// Copyright Claudio Mattera 2023.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for fetching data for temporal heatmap charts

use std::collections::HashMap;

use tracing::debug;

use miette::miette;
use miette::{IntoDiagnostic, Report, WrapErr};

use chrono::{DateTime, Utc};

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

use crate::configuration::TemporalHeatMapConfiguration;

/// Fetch data for temporal heatmap
///
/// # Errors
///
/// Return and error when data could not be fetched
pub async fn fetch_data(
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
