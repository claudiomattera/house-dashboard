// Copyright Claudio Mattera 2023.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for fetching data for trend charts

use std::collections::HashMap;
use std::fmt::Error as FmtError;
use std::fmt::Write;

use tracing::debug;

use miette::{IntoDiagnostic, Report, WrapErr};

use time::Duration;

use chrono::{DateTime, Utc};

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

use crate::configuration::TrendConfiguration;

/// Fetch data for trend
///
/// # Errors
///
/// Return and error when data could not be fetched
pub async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    trend_configuration: &TrendConfiguration,
) -> Result<HashMap<String, Vec<(DateTime<Utc>, f64)>>, Report> {
    let query = format!(
        "SELECT {scale} * {aggregator}({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago}
        GROUP BY time({period}),{tag} FILL(none)",
        database = trend_configuration.database,
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
