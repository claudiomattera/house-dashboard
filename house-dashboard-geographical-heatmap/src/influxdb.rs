// Copyright Claudio Mattera 2023.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for fetching data for geographical heatmap charts

use std::collections::HashMap;
use std::fmt::Error as FmtError;
use std::fmt::Write;

use tracing::debug;

use miette::{IntoDiagnostic, Report, WrapErr};

use time::Duration;

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

use crate::configuration::GeographicalHeatMapConfiguration;

/// Fetch data for geographical heatmap
///
/// # Errors
///
/// Return and error when data could not be fetched
pub async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    geographical_heatmap_configuration: &GeographicalHeatMapConfiguration,
) -> Result<HashMap<String, Option<f64>>, Report> {
    let query = format!(
        "SELECT {scale} * last({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago}
        GROUP BY {tag} FILL(none)",
        database = geographical_heatmap_configuration.database,
        scale = geographical_heatmap_configuration.scale.unwrap_or(1.0),
        field = geographical_heatmap_configuration.field,
        measurement = geographical_heatmap_configuration.measurement,
        tag = geographical_heatmap_configuration.tag,
        how_long_ago = duration_to_query(&geographical_heatmap_configuration.how_long_ago.duration)
            .into_diagnostic()?,
    );

    debug!("Query: {}", query);

    let time_seriess = match influxdb_client
        .fetch_tagged_dataframes(&query, &geographical_heatmap_configuration.tag)
        .await
    {
        Ok(time_seriess) => Ok(time_seriess),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch time-series")?;

    let values = time_seriess
        .into_iter()
        .map(|(region, ts)| {
            (
                region,
                ts.last().map(|&(ref _instant, ref value)| value).copied(),
            )
        })
        .collect::<HashMap<String, Option<f64>>>();

    Ok(values)
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
