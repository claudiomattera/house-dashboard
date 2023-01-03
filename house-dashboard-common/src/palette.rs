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
///
/// 1. `#000000` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAGUlEQVR4nGNgGAWjYBSMglEwCkbBKBgmAAAI2QABplD5BgAAAABJRU5ErkJggg==">
/// 2. `#ffffff` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAzDsK78OWcE9vhaDguBTpLps78Db7YIW4QtwhZhi7BFlLYu+ZEDG5bHFr0AAAAASUVORK5CYII=">
/// 3. `#202020` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK5Iwh/lCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXd5dAH7tkwRWAAAAAElFTkSuQmCC">
/// 4. `#c0c0c0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78QQXaCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXXJ8Al4qg2yLAAAAAElFTkSuQmCC">
/// 5. `#808080` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAzDsK7IA30E9vhaDguBTpLps78Db7YIW4QtwhZhi7BFlLYu0NMBns/lgQAAAAAASUVORK5CYII=">
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
///
/// 1. `#ffffff` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAzDsK78OWcE9vhaDguBTpLps78Db7YIW4QtwhZhi7BFlLYu+ZEDG5bHFr0AAAAASUVORK5CYII=">
/// 2. `#000000` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAGUlEQVR4nGNgGAWjYBSMglEwCkbBKBgmAAAI2QABplD5BgAAAABJRU5ErkJggg==">
/// 3. `#c0c0c0` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78QQXaCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXXJ8Al4qg2yLAAAAAElFTkSuQmCC">
/// 4. `#202020` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK5Iwh/lCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXd5dAH7tkwRWAAAAAElFTkSuQmCC">
/// 5. `#808080` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAzDsK7IA30E9vhaDguBTpLps78Db7YIW4QtwhZhi7BFlLYu0NMBns/lgQAAAAAASUVORK5CYII=">
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
///
/// 1. `#e41a1c` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8IiXDMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLABLxAE4DctCAAAAAABJRU5ErkJggg==">
/// 2. `#377eb8` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM0r9vBMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLAAoXQGLmyCCWAAAAABJRU5ErkJggg==">
/// 3. `#4daf4a` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP0Xe/FMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLADQAgFkN3k5AgAAAABJRU5ErkJggg==">
/// 4. `#984ea3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGOc4beYYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQAgZgGnT9p17QAAAABJRU5ErkJggg==">
/// 5. `#ff7f00` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAL0lEQVR4nO3OMQEAMAjAMMD4pA8DPL3gaBQk/4uDajsws0XYImwRtghbhC3iaKsBwCQBnK3rjfUAAAAASUVORK5CYII=">
/// 6. `#ffff33` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/9+YYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQDuoAJPB8UZggAAAABJRU5ErkJggg==">
/// 7. `#a65628` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNcFqbBMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACj4AFC/66IzAAAAABJRU5ErkJggg==">
/// 8. `#f781bf` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP83rifYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQAjJQJVXlMunwAAAABJRU5ErkJggg==">
/// 9. `#999999` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78CYTtCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXWf9Aeke9BM0AAAAAElFTkSuQmCC">
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
///
/// 1. `#66c2a5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNMO7SUYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQB5bAHrxEqegAAAAABJRU5ErkJggg==">
/// 2. `#fc8d62` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP805vEMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACDhgIJwUq/iwAAAABJRU5ErkJggg==">
/// 3. `#8da0cb` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPsXXCaYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQD1mQIWi/O0QAAAAABJRU5ErkJggg==">
/// 4. `#e78ac3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN83nWYYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQAIjAJSckV8ggAAAABJRU5ErkJggg==">
/// 5. `#a6d854` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNcdiOEYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQCmMAHwAorxxQAAAABJRU5ErkJggg==">
/// 6. `#ffd92f` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8f1OfYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQB7TwIldT2R2gAAAABJRU5ErkJggg==">
/// 7. `#e5c494` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGN8emQKw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswBYSAJbZjR42wAAAABJRU5ErkJggg==">
/// 8. `#b3b3b3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK782YTgCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXRmsAjeK0gVXAAAAAElFTkSuQmCC">
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
///
/// 1. `#8dd3c7` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPsvXycYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQCVNAJFeSFOKQAAAABJRU5ErkJggg==">
/// 2. `#ffffb3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/38zw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswBZ6wLP+kvinQAAAABJRU5ErkJggg==">
/// 3. `#bebada` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPct+sWw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswARjQJwF57dOwAAAABJRU5ErkJggg==">
/// 4. `#fb8072` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP83VDEMPgA00A7ADsYdRYpYNRZpIBRZ5ECRp1FChh1Filg1FmkgEHqLACVIwILViwvRgAAAABJRU5ErkJggg==">
/// 5. `#80b1d3` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGNs2HiZYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQBfpwIi4eIk+gAAAABJRU5ErkJggg==">
/// 6. `#fdb462` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8uyWJYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQDlLgIxdZA/lgAAAABJRU5ErkJggg==">
/// 7. `#b3de69` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPcfC+TYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQAH3gIYhi7AkQAAAABJRU5ErkJggg==">
/// 8. `#fccde5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8c/Ypw+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswA/MQLMCDPizgAAAABJRU5ErkJggg==">
/// 9. `#d9d9d9` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMElEQVR4nO3OMQEAMAzDsK78CYbNCOzxtRwWAp0k02d/B95sEbYIW4QtwhZhiyhtXQmmAqnZ9uNPAAAAAElFTkSuQmCC">
/// 10. `#bc80bd` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGPc07CXYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQD+rQIXh9TJJAAAAABJRU5ErkJggg==">
/// 11. `#ccebc5` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGM88/oow+ADTAPtAOxg1FmkgFFnkQJGnUUKGHUWKWDUWaSAUWeRAgapswCFBQKa0i+lmwAAAABJRU5ErkJggg==">
/// 12. `#ffed6f` <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAPCAIAAAApuQuVAAAAMUlEQVR4nGP8/zafYfABpoF2AHYw6ixSwKizSAGjziIFjDqLFDDqLFLAqLNIAYPUWQBhyAJ5lFLTqQAAAABJRU5ErkJggg==">
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
