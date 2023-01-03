// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for generating chart

use tracing::info;

use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    drawing::IntoDrawingArea,
    element::BitMapElement,
};

use image::open as open_image;
use image::RgbImage;

use house_dashboard_common::{configuration::StyleConfiguration, palette::SystemColor};

use crate::Error;
use crate::ImageConfiguration;

/// Draw an image chart
///
/// # Errors
///
/// Return and error when chart generation failed
pub fn draw_image(
    image: &ImageConfiguration,
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error> {
    info!("Drawing image '{}'", image.path.display());

    let root = backend.into_drawing_area();
    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    let image: RgbImage = open_image(&image.path)?.to_rgb8();

    let width = image.width();
    let height = image.height();
    let raw: Vec<u8> = image.into_vec();

    let mut bitmap_element: BitMapElement<(i32, i32)> = BitMapElement::new((0, 0), (width, height));
    bitmap_element
        .as_bitmap_backend()
        .blit_bitmap((0, 0), (width, height), &raw)?;

    root.draw(&bitmap_element)?;

    Ok(())
}
