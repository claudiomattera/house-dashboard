// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for generating infrastructure summary charts

#![allow(clippy::module_name_repetitions)]

use tracing::instrument;

use miette::{Report, WrapErr};

use house_dashboard_common::configuration::StyleConfiguration;
use plotters::backend::BitMapBackend;

mod chart;
pub use self::chart::draw_image;

mod configuration;
pub use self::configuration::ImageConfiguration;

mod error;
pub use self::error::Error;

/// Fetch data and draw chart for infrastructure summary
///
/// # Errors
///
/// Return and error when chart generation failed
#[allow(clippy::unreachable)]
#[instrument(name = "image", skip(image_configuration, style_configuration))]
pub async fn process_image(
    image_configuration: &ImageConfiguration,
    style_configuration: &StyleConfiguration,
    index: usize,
) -> Result<Vec<u8>, Report> {
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);
    draw_image(image_configuration, style_configuration, backend)
        .wrap_err("cannot draw infrastructure summary")?;

    Ok(buffer)
}
