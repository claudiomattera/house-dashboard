// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for colormaps

use serde::Deserialize;

use palette::{Gradient, LinSrgb, Pixel, Srgb};

use plotters::style::{Color, RGBAColor, RGBColor};

/// Type of color maps
#[derive(Debug, Deserialize)]
pub enum ColormapType {
    /// Cool/Warm
    CoolWarm,

    /// Shades of blue
    Blues,

    /// Shades of red
    Reds,

    /// Shades of green
    Greens,

    /// Shades of orange
    Oranges,

    /// Shades of violet
    Violets,

    /// Shades of gray
    Grays,

    /// Status (ok/warning/error)
    Status,
}

/// A Color map
#[derive(Debug)]
pub struct Colormap {
    /// Color map gradient
    gradient: Gradient<LinSrgb<f64>>,

    /// Gradient lower bound
    min: f64,

    /// Gradient upper bound
    max: f64,
}

impl Colormap {
    /// Create a new colormap with bounds and direction
    #[must_use]
    pub fn new_with_bounds_and_direction(
        colormap_type: Option<&ColormapType>,
        min: f64,
        max: f64,
        reversed: Option<bool>,
    ) -> Self {
        let base_palette = match *colormap_type.unwrap_or(&ColormapType::Blues) {
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
            base_palette.iter().rev().copied().collect::<Vec<[u8; 3]>>()
        } else {
            base_palette.to_vec()
        };
        let linear_palette = palette.iter().map(|&[r, g, b]| {
            Srgb::new(
                f64::from(r) / 255.0,
                f64::from(g) / 255.0,
                f64::from(b) / 255.0,
            )
            .into_linear()
        });
        let gradient = Gradient::new(linear_palette);
        Colormap { gradient, min, max }
    }

    /// Create a new colormap with bounds
    #[must_use]
    pub fn new_with_bounds(colormap_type: Option<&ColormapType>, min: f64, max: f64) -> Self {
        Self::new_with_bounds_and_direction(colormap_type, min, max, None)
    }

    /// Map value to color
    #[must_use]
    pub fn get_color(&self, value: f64) -> RGBColor {
        let [r, g, b] = self.get_color_array(value);
        RGBColor(r, g, b)
    }

    /// Map value to RGB array
    #[must_use]
    pub fn get_color_array(&self, value: f64) -> [u8; 3] {
        let value = if value.is_nan() {
            self.min
        } else {
            (value - self.min) / (self.max - self.min)
        };
        let color = self.gradient.get(value);
        let pixel: [u8; 3] = Srgb::from_linear(color).into_format().into_raw();
        pixel
    }
}

/// Palette from shades of red
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#fff5f0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8//UDw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswAcoQMC4kmIGwAAAABJRU5ErkJggg==">
/// 2. `#fee0d2` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP89+ASw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswBQ9ALObkG0fAAAAABJRU5ErkJggg==">
/// 3. `#fcbba1` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8s3shw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswBHDgJ2w5jadgAAAABJRU5ErkJggg==">
/// 4. `#fc9272` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8M6mIYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQA9KAIem2wI4gAAAABJRU5ErkJggg==">
/// 5. `#fb6a4a` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8neXFMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABxGwHNGrswVwAAAABJRU5ErkJggg==">
/// 6. `#ef3b2c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8b63DMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABeQQF0DRDTUQAAAABJRU5ErkJggg==">
/// 7. `#cb181d` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM8LSHLMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABlxQEei0YnhgAAAABJRU5ErkJggg==">
/// 8. `#a50f15` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNcyi/KMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAB/WADnIRCsCAAAAABJRU5ErkJggg==">
/// 9. `#67000d` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNMZ+BlGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgCPkgCS7iTDawAAAABJRU5ErkJggg==">
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

/// Palette from shades of blue
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#f7fbff` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/vs/w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswCPdQMP27Oz/AAAAABJRU5ErkJggg==">
/// 2. `#deebf7` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO89/o7w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswDeHwLefw5Q+QAAAABJRU5ErkJggg==">
/// 3. `#c6dbef` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM8dvs9w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswA1sAKueCnVwwAAAABJRU5ErkJggg==">
/// 4. `#9ecae1` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGOcd+ohw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswDByAJnH/PDjQAAAABJRU5ErkJggg==">
/// 5. `#6baed6` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPMXneNYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQCl3QINCcJ6FgAAAABJRU5ErkJggg==">
/// 6. `#4292c6` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN0mnSMYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQC2NAG4JA/bDgAAAABJRU5ErkJggg==">
/// 7. `#2171b5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNULNzKMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLADYQgFl/J4eawAAAABJRU5ErkJggg==">
/// 8. `#08519c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPkCJzDMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAADRgETghU3nAAAAABJRU5ErkJggg==">
/// 9. `#08306b` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPkMMhmGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgAubADBss5bQQAAAABJRU5ErkJggg==">
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

/// Palette from shades of green
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#f7fcf5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/ucrw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswA/8AMG2ESTCAAAAABJRU5ErkJggg==">
/// 2. `#e5f5e0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8+vUBw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswCpMwLY1t4qyQAAAABJRU5ErkJggg==">
/// 3. `#c7e9c0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM8/vIAw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswAa8QKOk98DZwAAAABJRU5ErkJggg==">
/// 4. `#a1d99b` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNceHM2w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswD2RwIzSIJlKgAAAABJRU5ErkJggg==">
/// 5. `#74c476` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGMsOVLGMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABnkQHMwsQAkQAAAABJRU5ErkJggg==">
/// 6. `#41ab5d` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN0XB3LMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLADqaAFnAG8rUgAAAABJRU5ErkJggg==">
/// 7. `#238b45` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNU7nZlGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgDx+wERwS53tgAAAABJRU5ErkJggg==">
/// 8. `#006d2c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNkyNVhGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgDWLgC3B5iPDAAAAABJRU5ErkJggg==">
/// 9. `#00441b` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNkcJFmGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgDVawB9d2D0BQAAAABJRU5ErkJggg==">
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

/// Palette from shades of gray
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#ffffff` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAzDsK78OWcE9vhaDguBTpLps78Db7YIW4QtwhZhi7BFlLYu+ZEDG5bHFr0AAAAASUVORK5CYII=">
/// 2. `#f0f0f0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78wQbCCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXWu3Au6b4OcZAAAAAElFTkSuQmCC">
/// 3. `#d9d9d9` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78CYbNCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXQmmAqnZ9uNPAAAAAElFTkSuQmCC">
/// 4. `#bdbdbd` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78KYXbCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXSLtAlX7lw0yAAAAAElFTkSuQmCC">
/// 5. `#969696` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OoQEAMAzDsK7/49y7B0aMFmBdoJNk+uzvwJstwhZhi7BF2CJsEaWtCxhuAeAqIiopAAAAAElFTkSuQmCC">
/// 6. `#737373` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OsQ0AMAzDsDQv+3+gD3TRVA/iBTxJps/+DrzZImwRtghbhC3CFlHaungDAXdIKrZEAAAAAElFTkSuQmCC">
/// 7. `#525252` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK4Qwx/MCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXQyxARQqIrTAAAAAAElFTkSuQmCC">
/// 8. `#252525` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK5cwp/iCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXWMFAI3rnU30AAAAAElFTkSuQmCC">
/// 9. `#000000` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAGUlEQVR4nGNgGAWjYBSMglEwCkbBKBgmAAAI2QABplD5BgAAAABJRU5ErkJggg==">
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

/// Palette from shades of orange
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#fff5eb` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8//U1w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswDwZAL9sd5EvAAAAABJRU5ErkJggg==">
/// 2. `#fee6ce` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP89+wcw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswBipgLQn9HqcgAAAABJRU5ErkJggg==">
/// 3. `#fdd0a2` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8e2ERw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswASbgKNhxm0kQAAAABJRU5ErkJggg==">
/// 4. `#fdae6b` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8uy6bYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQD/qgI0DBTViQAAAABJRU5ErkJggg==">
/// 5. `#fd8d3c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP822vDMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAA8iwHkpSj8IgAAAABJRU5ErkJggg==">
/// 6. `#f16913` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8mCnMMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAApvAGLY8kTjwAAAABJRU5ErkJggg==">
/// 7. `#d94801` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO86cHIMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACSjAFA67yuIQAAAABJRU5ErkJggg==">
/// 8. `#a63603` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNcZsbMMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABB9AD9bYRLoQAAAABJRU5ErkJggg==">
/// 9. `#7f2704` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGOsV2dhGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgBtKwDIxgUGvQAAAABJRU5ErkJggg==">
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

/// Palette from shades of violet
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#fcfbfd` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP88/svw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswCqAQMSigPoiwAAAABJRU5ErkJggg==">
/// 2. `#efedf5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8//Yrw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswB0iALvGRuWqgAAAABJRU5ErkJggg==">
/// 3. `#dadaeb` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO8des1w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswC6YQK9Mfo2PAAAAABJRU5ErkJggg==">
/// 4. `#bcbddc` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPcs/cOw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswAsDgJznvvNwAAAAABJRU5ErkJggg==">
/// 5. `#9e9ac8` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGOcN+sEw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswA8dAIeTBavOgAAAABJRU5ErkJggg==">
/// 6. `#807dba` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNsqN3FMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAC26AHVnyE3fQAAAABJRU5ErkJggg==">
/// 7. `#6a51a3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPMClzMMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACj/QF86da/owAAAABJRU5ErkJggg==">
/// 8. `#54278f` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGMMUe9nGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgC9QgEoxMm+wAAAAABJRU5ErkJggg==">
/// 9. `#3f007d` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO0Z6hlGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgALnwDapUfqmwAAAABJRU5ErkJggg==">
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

/// Palette from shades from cool to warm
///
/// From <https://colorbrewer2.org/>
///
/// 1. `#053061` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNkNUhkGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgC7eQC0U1KszQAAAABJRU5ErkJggg==">
/// 2. `#2166ac` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNUTFvDMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAAnfwFRscHL2wAAAABJRU5ErkJggg==">
/// 3. `#0571b0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNkLdzAMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAC0ZQFE7C+IVwAAAABJRU5ErkJggg==">
/// 4. `#4393c3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN0nnyYYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQCtYQG3XQMzYwAAAABJRU5ErkJggg==">
/// 5. `#67a9cf` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNMX3meYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQAYcAH9RlqK8QAAAABJRU5ErkJggg==">
/// 6. `#92c5de` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGOcdPQew+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswAQ8wJTcNa2fgAAAABJRU5ErkJggg==">
/// 7. `#d1e5f0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO8+PQDw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswD4NALE9V0VvAAAAABJRU5ErkJggg==">
/// 8. `#f7f7f7` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OoQEAMAzDsK7//xu6B0aMFmBdoJNk+uzvwJstwhZhi7BF2CJsEaWtCyVpAwOCWPHYAAAAAElFTkSuQmCC">
/// 9. `#fddbc7` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8e/s4w+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswC6qAK9/iCnwwAAAABJRU5ErkJggg==">
/// 10. `#f4a582` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8srSJYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQArzAI508afaAAAAABJRU5ErkJggg==">
/// 11. `#ef8a62` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN835XEMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAD1+gH5nEPKTwAAAABJRU5ErkJggg==">
/// 12. `#d6604d` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGO8luDLMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLADr4QGhoUQbaQAAAABJRU5ErkJggg==">
/// 13. `#ca0020` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM8xaDAMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACjOAEIdNCjlQAAAABJRU5ErkJggg==">
/// 14. `#b2182b` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPcJKHNMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAAEYQETEKDXEAAAAABJRU5ErkJggg==">
/// 15. `#67001f` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNMZ5BnGHyAaaAdgB2MOosUMOosUsCos0gBo84iBYw6ixQw6ixSwCB1FgAurQCkdbnLowAAAABJRU5ErkJggg==">
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

/// Palette for status (ok / warning / error)
///
/// 1. `#4daf4a` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP0Xe/FMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLADQAgFkN3k5AgAAAABJRU5ErkJggg==">
/// 2. `#ffff33` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/9+YYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQDuoAJPB8UZggAAAABJRU5ErkJggg==">
/// 3. `#e41a1c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8IiXDMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABLxAE4DctCAAAAAABJRU5ErkJggg==">
const PALETTE_STATUS: &[[u8; 3]] = &[[77, 175, 74], [255, 255, 51], [228, 26, 28]];

/// Interpolate two colors
#[must_use]
pub fn interpolate_colors(c1: RGBAColor, c2: RGBColor) -> RGBAColor {
    let (r1, g1, b1) = c1.rgb();
    let (r2, g2, b2) = c2.rgb();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let r = (f32::from(r1) * 0.5 + f32::from(r2) * 0.5) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let g = (f32::from(g1) * 0.5 + f32::from(g2) * 0.5) as u8;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let b = (f32::from(b1) * 0.5 + f32::from(b2) * 0.5) as u8;

    RGBAColor(r, g, b, c1.alpha())
}
