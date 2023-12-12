// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating Proxmox summary charts

#![allow(clippy::module_name_repetitions)]

use tracing::instrument;

use miette::{Report, WrapErr};

use house_dashboard_influxdb::InfluxDBClient;

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_proxmox_summary;

mod configuration;
pub use self::configuration::ProxmoxSummaryConfiguration;

mod error;
pub use self::error::Error;

mod influxdb;
use self::influxdb::fetch_data;

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
