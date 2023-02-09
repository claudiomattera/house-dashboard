// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing command-line arguments

use std::path::{Path, PathBuf};

use bpaf::{construct, short, Parser};

/// Command-line arguments
#[derive(Debug, Clone)]
pub struct Arguments {
    /// Verbosity level
    pub verbosity: usize,

    /// Path to configuration directory
    pub configuration_directory_path: PathBuf,

    /// Path to output directory
    pub output_directory_path: PathBuf,
}

/// Parse command-line arguments
pub fn parse_command_line() -> Arguments {
    let verbosity = short('v')
        .long("verbose")
        .help("Verbosity level")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 5, "It doesn't get any more verbose than this");

    let configuration_directory_path = short('c')
        .long("configuration-directory")
        .help("Path to configuration directory")
        .argument::<PathBuf>("PATH");

    let output_directory_path = short('o')
        .long("output-directory")
        .help("Path to output directory (default: .)")
        .argument::<PathBuf>("PATH")
        .fallback(Path::new(".").to_owned());

    let parser = construct!(Arguments {
        verbosity,
        configuration_directory_path,
        output_directory_path
    })
    .to_options()
    .descr("Create dashboard images");

    parser.run()
}
