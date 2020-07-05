// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::fmt;

use palette::{Gradient, LinSrgb, Pixel, Srgb};

use plotters::style::RGBColor;

use crate::error::DashboardError;

#[derive(Debug)]
pub struct Colormap {
    name: String,
    gradient: Gradient<LinSrgb<f64>>,
    min: f64,
    max: f64,
}

impl Colormap {
    pub fn new_with_bounds(name: &str, min: f64, max: f64) -> Result<Self, DashboardError> {
        let base_palette = get_base_palette(name)?;
        let linear_palette = base_palette.iter().map(
            |[r, g, b]| Srgb::new(
                *r as f64 / 255.0,
                *g as f64 / 255.0,
                *b as f64 / 255.0,
            ).into_linear()
        );
        let gradient = Gradient::new(linear_palette);
        Ok(Colormap {
            name: name.to_owned(),
            gradient,
            min,
            max,
        })
    }

    pub fn get_color(self: &Self, value: f64) -> RGBColor {
        let [r, g, b] = self.get_color_array(value);
        RGBColor(r, g, b)
    }

    pub fn get_color_array(self: &Self, value: f64) -> [u8; 3] {
        let value = (value - self.min) / (self.max - self.min);
        let color = self.gradient.get(value);
        let pixel: [u8; 3] = Srgb::from_linear(color)
            .into_format()
            .into_raw();
        pixel
    }
}

impl fmt::Display for Colormap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn get_base_palette(name: &str) -> Result<&[[u8; 3]], DashboardError> {
    match name {
        "blues" => Ok(PALETTE_BLUES),
        "reds" => Ok(PALETTE_REDS),
        "coolwarm" => Ok(PALETTE_COOLWARM),
        _ => Err(DashboardError::UnknownPalette),
    }
}

// Palette from https://colorbrewer2.org/
const PALETTE_REDS: &[[u8; 3]] = &[
    [255, 245, 240],
    [254, 224, 210],
    [252, 187, 161],
    [252, 146, 114],
    [251, 106, 74],
    [239, 59, 44],
    [203, 24, 29],
    [165, 15, 21],
    [103, 0, 13],
];

// Palette from https://colorbrewer2.org/
const PALETTE_BLUES: &[[u8; 3]] = &[
    [247, 251, 255],
    [222, 235, 247],
    [198, 219, 239],
    [158, 202, 225],
    [107, 174, 214],
    [66, 146, 198],
    [33, 113, 181],
    [8, 81, 156],
    [8, 48, 107],
];

// Palette from https://colorbrewer2.org/
const PALETTE_COOLWARM: &[[u8; 3]] = &[
    [5, 48, 97],
    [33, 102, 172],
    [5, 113, 176],
    [67, 147, 195],
    [103, 169, 207],
    [146, 197, 222],
    [209, 229, 240],
    [247, 247, 247],
    [253, 219, 199],
    [244, 165, 130],
    [239, 138, 98],
    [214, 96, 77],
    [202, 0, 32],
    [178, 24, 43],
    [103, 0, 31],
];
