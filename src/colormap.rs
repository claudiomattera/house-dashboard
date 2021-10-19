// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use serde::Deserialize;

use palette::{Gradient, LinSrgb, Pixel, Srgb};

use plotters::style::RGBColor;

#[derive(Debug, Deserialize)]
pub enum ColormapType {
    CoolWarm,
    Blues,
    Reds,
    Greens,
    Oranges,
    Violets,
    Grays,
    Status,
}

#[derive(Debug)]
pub struct Colormap {
    gradient: Gradient<LinSrgb<f64>>,
    min: f64,
    max: f64,
}

impl Colormap {
    pub fn new_with_bounds_and_direction(
        colormap_type: Option<&ColormapType>,
        min: f64,
        max: f64,
        reversed: Option<bool>,
    ) -> Self {
        let base_palette = match colormap_type.unwrap_or(&ColormapType::Blues) {
            ColormapType::CoolWarm => PALETTE_COOLWARM,
            ColormapType::Reds => PALETTE_REDS,
            ColormapType::Blues => PALETTE_BLUES,
            ColormapType::Greens => PALETTE_GREENS,
            ColormapType::Oranges => PALETTE_ORANGES,
            ColormapType::Violets => PALETTE_VIOLETS,
            ColormapType::Grays => PALETTE_GRAYS,
            ColormapType::Status => PALETTE_STATUS,
        };
        let palette = if let Some(true) = reversed {
            base_palette.iter().rev().cloned().collect::<Vec<[u8; 3]>>()
        } else {
            base_palette.to_vec()
        };
        let linear_palette = palette.iter().map(|[r, g, b]| {
            Srgb::new(*r as f64 / 255.0, *g as f64 / 255.0, *b as f64 / 255.0).into_linear()
        });
        let gradient = Gradient::new(linear_palette);
        Colormap { gradient, min, max }
    }

    pub fn new_with_bounds(colormap_type: Option<&ColormapType>, min: f64, max: f64) -> Self {
        Self::new_with_bounds_and_direction(colormap_type, min, max, None)
    }

    pub fn get_color(&self, value: f64) -> RGBColor {
        let [r, g, b] = self.get_color_array(value);
        RGBColor(r, g, b)
    }

    pub fn get_color_array(&self, value: f64) -> [u8; 3] {
        let value = (value - self.min) / (self.max - self.min);
        let color = self.gradient.get(value);
        let pixel: [u8; 3] = Srgb::from_linear(color).into_format().into_raw();
        pixel
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
const PALETTE_GREENS: &[[u8; 3]] = &[
    [247, 252, 245],
    [229, 245, 224],
    [199, 233, 192],
    [161, 217, 155],
    [116, 196, 118],
    [65, 171, 93],
    [35, 139, 69],
    [0, 109, 44],
    [0, 68, 27],
];

// Palette from https://colorbrewer2.org/
const PALETTE_GRAYS: &[[u8; 3]] = &[
    [255, 255, 255],
    [240, 240, 240],
    [217, 217, 217],
    [189, 189, 189],
    [150, 150, 150],
    [115, 115, 115],
    [82, 82, 82],
    [37, 37, 37],
    [0, 0, 0],
];

// Palette from https://colorbrewer2.org/
const PALETTE_ORANGES: &[[u8; 3]] = &[
    [255, 245, 235],
    [254, 230, 206],
    [253, 208, 162],
    [253, 174, 107],
    [253, 141, 60],
    [241, 105, 19],
    [217, 72, 1],
    [166, 54, 3],
    [127, 39, 4],
];

// Palette from https://colorbrewer2.org/
const PALETTE_VIOLETS: &[[u8; 3]] = &[
    [252, 251, 253],
    [239, 237, 245],
    [218, 218, 235],
    [188, 189, 220],
    [158, 154, 200],
    [128, 125, 186],
    [106, 81, 163],
    [84, 39, 143],
    [63, 0, 125],
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

#[rustfmt::skip]
const PALETTE_STATUS: &[[u8; 3]] = &[
    [77, 175, 74],
    [255, 255, 51],
    [228, 26, 28],
];
