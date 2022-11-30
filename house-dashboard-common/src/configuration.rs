// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use std::path::PathBuf;

use serde::Deserialize;

use crate::palette::{SeriesPalette, SystemPalette};

#[derive(Debug, Deserialize)]
pub struct StyleConfiguration {
    pub font_name: String,
    pub font_path: PathBuf,
    pub font_scale: f64,
    pub system_palette: SystemPalette,
    pub series_palette: SeriesPalette,
    pub draw_markers: Option<bool>,
    pub resolution: (u32, u32),
}
