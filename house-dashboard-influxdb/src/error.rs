// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for handling errors

use std::io::Error as IOError;

use thiserror::Error as ThisError;

use serde_json::Error as SerdeJsonError;

use serde_html_form::ser::Error as SerdeFormError;

use isahc::Error as IsahcError;

use isahc::http::Error as HttpError;

use url::ParseError as UrlParseError;

/// An error occurred generating a chart
#[derive(ThisError, Debug)]
pub enum Error {
    /// Unknown error
    #[error("Unknown error")]
    Unknown,

    /// InfluxDB returned empty results
    #[error("InfluxDB returned empty results")]
    EmptyInfluxDBResults,

    /// InfluxDB returned an empty series
    #[error("InfluxDB returned an empty series")]
    EmptySeries,

    /// InfluxDB did not return a series of tags
    #[error("InfluxDB did not return a series of tags ")]
    NotATagSeries,

    /// InfluxDB did not return a time-series
    #[error("InfluxDB did not return a time-series")]
    NotATimeSeries,

    /// InfluxDB returned no or empty tags
    #[error("InfluxDB returned no or empty tags")]
    EmptyTags,

    /// InfluxDB did not return a value for specified tag
    #[error("InfluxDB did not return a value for tag \"{0}\"")]
    MissingTag(String),

    /// InfluxDB returned an error
    #[error("InfluxDB returned error \"{0}\"")]
    InfluxDBError(String),

    /// JSON deserialization failure
    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),

    /// Form URL-encoded serialization failure
    #[error(transparent)]
    SerdeForm(#[from] SerdeFormError),

    /// ISAHC error
    #[error(transparent)]
    Isahc(#[from] IsahcError),

    /// ISAHC HTTP error
    #[error(transparent)]
    Http(#[from] HttpError),

    /// IO error
    #[error(transparent)]
    IO(#[from] IOError),

    /// URL parse error
    #[error(transparent)]
    UrlParse(#[from] UrlParseError),
}
