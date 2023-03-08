// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use std::path::PathBuf;

use serde::Deserialize;

/// Chart configuration for image charts
#[derive(Debug, Deserialize)]
pub struct ImageConfiguration {
    /// Image path
    pub path: PathBuf,
}
