// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use clap::Parser;

use std::path::PathBuf;

use chrono::{DateTime, ParseError, Utc};

#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Arguments {
    /// Increase verbosity (-v, -vv, -vvv, etc.)
    #[clap(short, long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,

    /// Path to configuration file
    #[clap(short, long = "configuration")]
    pub configuration_path: PathBuf,

    /// Create charts at a specific instant
    #[clap(short, long, parse(try_from_str = parse_datetime))]
    pub now: Option<DateTime<Utc>>,

    /// Subcommand
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Save charts to files
    Save {
        /// Path to charts directory
        #[clap(short = 'p', long = "path")]
        directory_path: PathBuf,

        /// Clears all .bmp files in charts directory
        #[clap(long)]
        clear: bool,
    },
}

fn parse_datetime(text: &str) -> Result<DateTime<Utc>, ParseError> {
    let datetime = DateTime::parse_from_rfc3339(text)?;
    let datetime = datetime.with_timezone(&Utc);
    Ok(datetime)
}
