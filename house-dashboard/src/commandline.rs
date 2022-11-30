// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing command-line arguments

use clap::{ArgAction, Parser};

use std::path::PathBuf;

/// Command-line arguments
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Verbosity level
    #[arg(short, long = "verbose", action = ArgAction::Count)]
    pub verbosity: u8,

    /// Path to configuration directory
    #[arg(short, long = "configuration-directory")]
    pub configuration_directory_path: PathBuf,
}
