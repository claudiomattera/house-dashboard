// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating trend charts

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

use chrono::{DateTime, TimeZone, Utc};

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_trend;

mod configuration;
pub use self::configuration::TrendConfiguration;

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for trend
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(name = "trend", skip(trend_configuration, style_configuration))]
pub async fn process_trend(
    trend_configuration: &TrendConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let time_seriess = fetch_data().await.wrap_err("cannot fetch data for trend")?;

    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_trend(
        trend_configuration,
        &time_seriess,
        style_configuration,
        backend,
    )
    .wrap_err("cannot draw trend")?;

    Ok(buffer)
}

/// Fetch data for trend
///
/// # Errors
///
/// Return and error when data could not be fetched
#[allow(clippy::unused_async)]
async fn fetch_data() -> Result<HashMap<String, Vec<(DateTime<Utc>, f64)>>, Report> {
    let mut time_seriess: HashMap<String, Vec<(DateTime<Utc>, f64)>> = HashMap::new();
    time_seriess.insert(
        "living room".to_owned(),
        vec![
            (Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(), 24.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 10, 10, 11).unwrap(), 26.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 11, 10, 11).unwrap(), 27.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 12, 10, 11).unwrap(), 25.0),
        ],
    );
    time_seriess.insert(
        "bedroom".to_owned(),
        vec![
            (Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 32).unwrap(), 23.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 11, 0, 32).unwrap(), 24.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 12, 0, 32).unwrap(), 20.0),
        ],
    );
    time_seriess.insert(
        "stairs".to_owned(),
        vec![
            (Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 32).unwrap(), 18.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 11, 0, 32).unwrap(), 19.0),
            (Utc.with_ymd_and_hms(2014, 7, 8, 12, 0, 32).unwrap(), 18.0),
        ],
    );

    Ok(time_seriess)
}
