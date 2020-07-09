// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::path::PathBuf;

use plotters::drawing::{BitMapBackend, DrawingBackend, IntoDrawingArea};
use plotters::element::BitMapElement;

use image::{open, RgbImage};

use crate::error::DashboardError;

pub fn draw_image(
            path: PathBuf,
            root: BitMapBackend,
        ) -> Result<(), DashboardError> {
    info!("Drawing image");

    let root = root.into_drawing_area();

    let image: RgbImage = open(&path)
        .map_err(|error| DashboardError::ImageError(error))?
        .to_rgb();

    let width = image.width();
    let height = image.height();
    let raw: Vec<u8> = image.into_vec();

    let mut bitmap_element: BitMapElement<(i32, i32)> = BitMapElement::new((0, 0), (width, height));
    bitmap_element
        .as_bitmap_backend()
        .blit_bitmap(
            (0, 0),
            (width, height),
            &raw,
        )?;

    root.draw(&bitmap_element)?;

    Ok(())
}
