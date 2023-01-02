// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating infrastructure summary charts

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

use time::OffsetDateTime;

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_infrastructure_summary;

mod configuration;
pub use self::configuration::InfrastructureSummaryConfiguration;

mod error;
pub use self::error::Error;

mod loadbar;
pub use self::loadbar::Loadbar;

/// Fetch data and draw chart for infrastructure summary
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "infrastructure_summary",
    skip(infrastructure_summary_configuration, style_configuration)
)]
pub async fn process_infrastructure_summary(
    infrastructure_summary_configuration: &InfrastructureSummaryConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let now = OffsetDateTime::now_utc();

    let (hosts, loads) = fetch_data()
        .await
        .wrap_err("cannot fetch data for infrastructure summary")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_infrastructure_summary(
        infrastructure_summary_configuration,
        now,
        &hosts,
        &loads,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw infrastructure summary")?;

    Ok(buffer)
}

/// Fetch data for infrastructure summary
///
/// # Errors
///
/// Return and error when data could not be fetched
#[allow(clippy::unused_async)]
async fn fetch_data() -> Result<(HashSet<String>, HashMap<String, f64>), Report> {
    let mut hosts: HashSet<String> = HashSet::new();
    hosts.insert("dashboard.dk.claudiomattera.it".to_owned());
    hosts.insert("h2plus.dk.claudiomattera.it".to_owned());
    hosts.insert("media-center.dk.claudiomattera.it".to_owned());
    hosts.insert("vps.de.claudiomattera.it".to_owned());

    let mut loads: HashMap<String, f64> = HashMap::new();
    loads.insert("dashboard.dk.claudiomattera.it".to_owned(), 0.2);
    loads.insert("media-center.dk.claudiomattera.it".to_owned(), 0.9);
    loads.insert("vps.de.claudiomattera.it".to_owned(), 0.1);

    Ok((hosts, loads))
}
