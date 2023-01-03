// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating geographical heatmap charts

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

use tracing::instrument;

use miette::{Report, WrapErr};

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_geographical_heatmap;

mod configuration;
pub use self::configuration::{GeographicalHeatMapConfiguration, GeographicalRegionConfiguration};

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for geographical heatmap
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "geographical_heatmap",
    skip(geographical_heatmap_configuration, style_configuration)
)]
pub async fn process_geographical_heatmap(
    geographical_heatmap_configuration: &GeographicalHeatMapConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let values = fetch_data()
        .await
        .wrap_err("cannot fetch data for geographical heatmap")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_geographical_heatmap(
        geographical_heatmap_configuration,
        &values,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw geographical heatmap")?;

    Ok(buffer)
}

/// Fetch data for geographical heatmap
///
/// # Errors
///
/// Return and error when data could not be fetched
#[allow(clippy::unused_async)]
async fn fetch_data() -> Result<HashMap<String, Option<f64>>, Report> {
    let mut values: HashMap<String, Option<f64>> = HashMap::new();
    values.insert("living room".to_owned(), Some(22.0));
    values.insert("bedroom".to_owned(), Some(20.0));
    values.insert("bathroom".to_owned(), Some(26.0));
    values.insert("kitchen".to_owned(), None);
    values.insert("entrance".to_owned(), None);

    Ok(values)
}
