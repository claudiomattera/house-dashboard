// Copyright Claudio Mattera 2023.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for fetching data for infrastructure summary charts

use std::collections::{HashMap, HashSet};
use std::fmt::Error as FmtError;
use std::fmt::Write;

use tracing::debug;

use miette::{IntoDiagnostic, Report, WrapErr};

use time::Duration;

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

/// Fetch data for infrastructure summary
///
/// # Errors
///
/// Return and error when data could not be fetched
pub async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    how_long_ago: &Duration,
) -> Result<(HashSet<String>, HashMap<String, f64>), Report> {
    let hosts: HashSet<String> = influxdb_client
        .fetch_tag_values("telegraf", "system", "host", "always-on", "true")
        .await
        .into_diagnostic()
        .wrap_err("cannot fetch existing hosts")?;

    let query = format!(
        "SELECT last({load_field}) / last({n_cpus_field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND \"{filter_tag_name}\" = '{filter_tag_value}'
        GROUP BY {tag}",
        load_field = "load15",
        n_cpus_field = "n_cpus",
        database = "telegraf",
        measurement = "system",
        tag = "host",
        filter_tag_name = "always-on",
        filter_tag_value = "true",
        how_long_ago = duration_to_query(how_long_ago).into_diagnostic()?,
    );

    debug!("Query: {}", query);

    let loads = match influxdb_client
        .fetch_tagged_dataframes(&query, "host")
        .await
    {
        Ok(loads) => Ok(loads),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch loads for always-on hosts")?;

    let loads: HashMap<String, f64> = loads
        .into_iter()
        .filter_map(|(name, series)| series.last().map(|&(_instant, ref value)| (name, *value)))
        .collect();

    Ok((hosts, loads))
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
