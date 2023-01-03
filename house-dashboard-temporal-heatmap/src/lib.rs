// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating temporal heatmap charts

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

use tracing::instrument;

use miette::{Report, WrapErr};

use chrono::{DateTime, TimeZone, Utc};

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_temporal_heatmap;

mod configuration;
pub use self::configuration::TemporalHeatMapConfiguration;

mod error;
pub use self::error::Error;

mod colorbar;
pub use self::colorbar::Colorbar;

/// Fetch data and draw chart for temporal heatmap
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(
    name = "temporal_heatmap",
    skip(temporal_heatmap_configuration, style_configuration)
)]
pub async fn process_temporal_heatmap(
    temporal_heatmap_configuration: &TemporalHeatMapConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let time_series = fetch_data()
        .await
        .wrap_err("cannot fetch data for temporal heatmap")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_temporal_heatmap(
        temporal_heatmap_configuration,
        &time_series,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw temporal heatmap")?;

    Ok(buffer)
}

/// Fetch data for temporal heatmap
///
/// # Errors
///
/// Return and error when data could not be fetched
#[allow(clippy::unused_async)]
async fn fetch_data() -> Result<Vec<(DateTime<Utc>, f64)>, Report> {
    let time_series = vec![
        (Utc.with_ymd_and_hms(2014, 7, 1, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 2, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 3, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 4, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 5, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 6, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 7, 9, 10, 11).unwrap(), 2.0),
        (Utc.with_ymd_and_hms(2014, 7, 8, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 9, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 10, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 11, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 12, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 13, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 14, 10, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 7, 15, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 16, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 17, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 18, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 19, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 20, 11, 10, 11).unwrap(), 12.0),
        (Utc.with_ymd_and_hms(2014, 7, 21, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 22, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 23, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 24, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 25, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 26, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 27, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 28, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 29, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 30, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 7, 31, 12, 10, 11).unwrap(), 20.0),
        (Utc.with_ymd_and_hms(2014, 8, 1, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 2, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 3, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 4, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 5, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 6, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 7, 9, 10, 11).unwrap(), 1.0),
        (Utc.with_ymd_and_hms(2014, 8, 8, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 9, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 10, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 11, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 12, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 13, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 14, 10, 10, 11).unwrap(), 3.0),
        (Utc.with_ymd_and_hms(2014, 8, 15, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 16, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 17, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 18, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 19, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 20, 11, 10, 11).unwrap(), 4.0),
        (Utc.with_ymd_and_hms(2014, 8, 21, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 22, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 23, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 24, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 25, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 26, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 27, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 28, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 29, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 30, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 8, 31, 12, 10, 11).unwrap(), 6.0),
        (Utc.with_ymd_and_hms(2014, 9, 1, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 2, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 3, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 4, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 5, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 6, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 7, 9, 10, 11).unwrap(), 7.0),
        (Utc.with_ymd_and_hms(2014, 9, 8, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 9, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 10, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 11, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 12, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 13, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 14, 10, 10, 11).unwrap(), 18.0),
        (Utc.with_ymd_and_hms(2014, 9, 15, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 16, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 17, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 18, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 19, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 20, 11, 10, 11).unwrap(), 31.0),
        (Utc.with_ymd_and_hms(2014, 9, 21, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 22, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 23, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 24, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 25, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 26, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 27, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 28, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 29, 12, 10, 11).unwrap(), 38.0),
        (Utc.with_ymd_and_hms(2014, 9, 30, 12, 10, 11).unwrap(), 38.0),
    ];

    Ok(time_series)
}
