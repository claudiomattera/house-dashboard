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

use tracing::instrument;

use miette::{Report, WrapErr};

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_proxmox_summary;

mod configuration;
pub use self::configuration::ProxmoxSummaryConfiguration;

mod error;
pub use self::error::Error;

mod loadbar;
pub use self::loadbar::Loadbar;

/// Fetch data and draw chart for Proxmox summary
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "proxmox_summary",
    skip(proxmox_summary_configuration, style_configuration)
)]
pub async fn process_proxmox_summary(
    proxmox_summary_configuration: &ProxmoxSummaryConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let (hosts, loads) = fetch_data()
        .await
        .wrap_err("cannot fetch data for Proxmox summary")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_proxmox_summary(
        proxmox_summary_configuration,
        &hosts,
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
#[allow(clippy::unused_async)]
async fn fetch_data() -> Result<(HashSet<String>, HashMap<String, f64>), Report> {
    let mut hosts: HashSet<String> = HashSet::new();
    hosts.insert("ntp.claudiomattera.it".to_owned());
    hosts.insert("dns.claudiomattera.it".to_owned());
    hosts.insert("dhcp.claudiomattera.it".to_owned());
    hosts.insert("ca.claudiomattera.it".to_owned());
    hosts.insert("ldap.claudiomattera.it".to_owned());
    hosts.insert("gotify.claudiomattera.it".to_owned());
    hosts.insert("mqtt.claudiomattera.it".to_owned());
    hosts.insert("nexus.claudiomattera.it".to_owned());
    hosts.insert("internal.claudiomattera.it".to_owned());
    hosts.insert("backup.claudiomattera.it".to_owned());
    hosts.insert("influxdb.claudiomattera.it".to_owned());
    hosts.insert("kapacitor.claudiomattera.it".to_owned());
    hosts.insert("chronograf.claudiomattera.it".to_owned());
    hosts.insert("git.claudiomattera.it".to_owned());
    hosts.insert("minio.claudiomattera.it".to_owned());
    hosts.insert("drone.claudiomattera.it".to_owned());
    hosts.insert("drone-runner-1.claudiomattera.it".to_owned());
    hosts.insert("torrent.claudiomattera.it".to_owned());
    hosts.insert("test.claudiomattera.it".to_owned());
    hosts.insert("matrix.claudiomattera.it".to_owned());
    hosts.insert("jellyfin.claudiomattera.it".to_owned());
    hosts.insert("photoprism.claudiomattera.it".to_owned());

    let mut loads: HashMap<String, f64> = HashMap::new();
    loads.insert("ntp.claudiomattera.it".to_owned(), 0.2);
    loads.insert("dns.claudiomattera.it".to_owned(), 0.9);
    loads.insert("dhcp.claudiomattera.it".to_owned(), 0.1);
    loads.insert("ca.claudiomattera.it".to_owned(), 0.16);
    loads.insert("ldap.claudiomattera.it".to_owned(), 0.43);
    loads.insert("gotify.claudiomattera.it".to_owned(), 0.43);
    loads.insert("mqtt.claudiomattera.it".to_owned(), 0.43);
    loads.insert("nexus.claudiomattera.it".to_owned(), 0.43);
    loads.insert("internal.claudiomattera.it".to_owned(), 0.43);
    loads.insert("backup.claudiomattera.it".to_owned(), 0.43);
    loads.insert("influxdb.claudiomattera.it".to_owned(), 0.3);
    loads.insert("kapacitor.claudiomattera.it".to_owned(), 0.3);
    loads.insert("chronograf.claudiomattera.it".to_owned(), 0.3);
    loads.insert("git.claudiomattera.it".to_owned(), 0.3);
    loads.insert("minio.claudiomattera.it".to_owned(), 0.3);
    loads.insert("drone.claudiomattera.it".to_owned(), 0.3);
    loads.insert("drone-runner-1.claudiomattera.it".to_owned(), 0.3);

    Ok((hosts, loads))
}
