// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use std::path::PathBuf;

use serde::Deserialize;

use crate::palette::{SeriesPalette, SystemPalette};

/// Style configuration
#[derive(Debug, Deserialize)]
pub struct StyleConfiguration {
    /// Font name
    pub font_name: String,

    /// Font path (relative to configuration directory)
    pub font_path: PathBuf,

    /// Font scale
    pub font_scale: f64,

    /// Palette for text and other controls
    pub system_palette: SystemPalette,

    /// Palette for charts
    pub series_palette: SeriesPalette,

    /// Flag for drawing marks
    pub draw_markers: Option<bool>,

    /// Chart resolution
    pub resolution: (u32, u32),
}
