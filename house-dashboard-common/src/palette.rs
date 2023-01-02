// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for palettes

use serde::Deserialize;

use plotters::style::Color;
use plotters::style::Palette;
use plotters::style::RGBColor;

/// Palette for text and other controls
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum SystemPalette {
    /// Dark palette
    Dark,

    /// Light palette
    Light,
}

/// Type of system color
#[derive(Clone, Copy, Debug)]
pub enum SystemColor {
    /// Background
    Background,

    /// Foreground
    Foreground,

    /// Light background
    LightBackground,

    /// Light foreground
    LightForeground,

    /// Middle
    Middle,
}

impl SystemColor {
    /// Get the index of a system color
    #[must_use]
    pub fn index(&self) -> usize {
        match *self {
            SystemColor::Background => 0,
            SystemColor::Foreground => 1,
            SystemColor::LightBackground => 2,
            SystemColor::LightForeground => 3,
            SystemColor::Middle => 4,
        }
    }
}

impl SystemPalette {
    /// Map a system color to a palette color
    #[must_use]
    pub fn pick(&self, color: SystemColor) -> RGBColor {
        match *self {
            SystemPalette::Dark => {
                let index = color.index();
                let (r, g, b) = PaletteDarkTheme::pick(index).rgb();
                RGBColor(r, g, b)
            }
            SystemPalette::Light => {
                let index = color.index();
                let (r, g, b) = PaletteLightTheme::pick(index).rgb();
                RGBColor(r, g, b)
            }
        }
    }
}

/// Palette for charts
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum SeriesPalette {
    /// Colorbrewer set 1
    ColorbrewerSet1,

    /// Colorbrewer set 2
    ColorbrewerSet2,

    /// Colorbrewer set 3
    ColorbrewerSet3,
}

impl SeriesPalette {
    /// Map a color to a palette color
    #[must_use]
    pub fn pick(&self, index: usize) -> RGBColor {
        match *self {
            SeriesPalette::ColorbrewerSet1 => {
                let (r, g, b) = PaletteColorbrewerSet1::pick(index).rgb();
                RGBColor(r, g, b)
            }
            SeriesPalette::ColorbrewerSet2 => {
                let (r, g, b) = PaletteColorbrewerSet2::pick(index).rgb();
                RGBColor(r, g, b)
            }
            SeriesPalette::ColorbrewerSet3 => {
                let (r, g, b) = PaletteColorbrewerSet3::pick(index).rgb();
                RGBColor(r, g, b)
            }
        }
    }
}

/// Wrapper palette for dark theme
pub struct PaletteDarkTheme;

impl Palette for PaletteDarkTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (0, 0, 0),
        (255, 255, 255),
        (32, 32, 32),
        (192, 192, 192),
        (128, 128, 128),
    ];
}

/// Wrapper palette for light theme
pub struct PaletteLightTheme;

impl Palette for PaletteLightTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (255, 255, 255),
        (0, 0, 0),
        (192, 192, 192),
        (32, 32, 32),
        (128, 128, 128),
    ];
}

/// Wrapper palette for Colorbrewer set 1
pub struct PaletteColorbrewerSet1;

impl Palette for PaletteColorbrewerSet1 {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (228, 26, 28),
        (55, 126, 184),
        (77, 175, 74),
        (152, 78, 163),
        (255, 127, 0),
        (255, 255, 51),
        (166, 86, 40),
        (247, 129, 191),
        (153, 153, 153),
    ];
}

/// Wrapper palette for Colorbrewer set 2
pub struct PaletteColorbrewerSet2;

impl Palette for PaletteColorbrewerSet2 {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (102, 194, 165),
        (252, 141, 98),
        (141, 160, 203),
        (231, 138, 195),
        (166, 216, 84),
        (255, 217, 47),
        (229, 196, 148),
        (179, 179, 179),
    ];
}

/// Wrapper palette for Colorbrewer set 3
pub struct PaletteColorbrewerSet3;

impl Palette for PaletteColorbrewerSet3 {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (141, 211, 199),
        (255, 255, 179),
        (190, 186, 218),
        (251, 128, 114),
        (128, 177, 211),
        (253, 180, 98),
        (179, 222, 105),
        (252, 205, 229),
        (217, 217, 217),
        (188, 128, 189),
        (204, 235, 197),
        (255, 237, 111),
    ];
}
