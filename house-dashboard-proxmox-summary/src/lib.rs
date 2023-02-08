// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating Proxmox summary charts

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

use std::collections::{HashMap, HashSet};
use std::fmt::Error as FmtError;
use std::fmt::Write;

use time::Duration;

use tracing::{debug, instrument};

use miette::{IntoDiagnostic, Report, WrapErr};

use house_dashboard_influxdb::Error as InfluxDBError;
use house_dashboard_influxdb::InfluxDBClient;

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_proxmox_summary;

mod configuration;
pub use self::configuration::ProxmoxSummaryConfiguration;

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for Proxmox summary
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "proxmox_summary",
    skip(influxdb_client, proxmox_summary_configuration, style_configuration)
)]
pub async fn process_proxmox_summary(
    influxdb_client: &InfluxDBClient,
    proxmox_summary_configuration: &ProxmoxSummaryConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let (hosts, statuses, loads) = fetch_data(
        influxdb_client,
        &proxmox_summary_configuration.node_fqdn,
        &proxmox_summary_configuration.how_long_ago.duration,
    )
    .await
    .wrap_err("cannot fetch data for Proxmox summary")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_proxmox_summary(
        proxmox_summary_configuration,
        &hosts,
        &statuses,
        &loads,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw Proxmox summary")?;

    Ok(buffer)
}

/// Fetch data for Proxmox summary
///
/// # Errors
///
/// Return and error when data could not be fetched
async fn fetch_data(
    influxdb_client: &InfluxDBClient,
    node_fqdn: &str,
    how_long_ago: &Duration,
) -> Result<(HashSet<String>, HashMap<String, String>, HashMap<String, f64>), Report> {
    let hosts: HashSet<String> = influxdb_client
        .fetch_tag_values("telegraf", "proxmox", "vm_name", "node_fqdn", node_fqdn)
        .await
        .into_diagnostic()
        .wrap_err("cannot fetch existing hosts")?;

    let load_query = format!(
        "SELECT last({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND \"{filter_tag_name}\" = '{filter_tag_value}'
        GROUP BY {tag}",
        field = "cpuload",
        database = "telegraf",
        measurement = "proxmox",
        tag = "vm_name",
        filter_tag_name = "node_fqdn",
        filter_tag_value = node_fqdn,
        how_long_ago = duration_to_query(how_long_ago).into_diagnostic()?,
    );

    debug!("Query: {}", load_query);

    let loads = match influxdb_client
        .fetch_tagged_dataframes(&load_query, "vm_name")
        .await
    {
        Ok(loads) => Ok(loads),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch loads for Proxmox VMs")?;

    let loads: HashMap<String, f64> = loads
        .into_iter()
        .filter_map(|(name, series)| series.last().map(|&(_instant, ref value)| (name, *value)))
        .collect();

    let status_query = format!(
        "SELECT last({field}) FROM {database}.autogen.{measurement}
        WHERE time < now() AND time > now() - {how_long_ago} AND \"{filter_tag_name}\" = '{filter_tag_value}'
        GROUP BY {tag}",
        field = "status",
        database = "telegraf",
        measurement = "proxmox",
        tag = "vm_name",
        filter_tag_name = "node_fqdn",
        filter_tag_value = node_fqdn,
        how_long_ago = duration_to_query(how_long_ago).into_diagnostic()?,
    );

    debug!("Query: {}", status_query);

    let statuses = match influxdb_client
        .fetch_tagged_string_dataframes(&status_query, "vm_name")
        .await
    {
        Ok(statuses) => Ok(statuses),
        Err(InfluxDBError::EmptySeries) => Ok(HashMap::new()),
        other => other,
    }
    .into_diagnostic()
    .wrap_err("cannot fetch status for Proxmox VMs")?;

    let statuses: HashMap<String, String> = statuses
        .into_iter()
        .filter_map(|(name, series)| series.last().map(|&(_instant, ref value)| (name, value.clone())))
        .collect();

    Ok((hosts, statuses, loads))
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
