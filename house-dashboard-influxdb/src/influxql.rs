// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for handling InfluxDB responses

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use serde::Deserialize;

use serde_json::from_slice as from_json_slice;
use serde_json::from_str as from_json_str;
use serde_json::Error as SerdeJsonError;
use serde_json::Value as JsonValue;

use chrono::{DateTime, Utc};

use house_dashboard_common::types::StringTimeSeries as OutputStringTimeSeries;
use house_dashboard_common::types::TimeSeries as OutputTimeSeries;

use crate::Error;

/// A tagged data-frame
///
/// This is defined as a newtype only to implement `TryFrom` on it.
#[derive(Debug, Clone)]
pub struct TaggedDataFrame(HashMap<String, OutputTimeSeries>);

impl TaggedDataFrame {
    /// Return an iterator over each named time-series
    pub fn iter(&self) -> impl Iterator<Item = (&String, &OutputTimeSeries)> {
        self.0.iter()
    }
}

#[allow(clippy::implicit_hasher)]
impl From<TaggedDataFrame> for HashMap<String, OutputTimeSeries> {
    fn from(dataframe: TaggedDataFrame) -> Self {
        dataframe.0
    }
}

/// A tagged string data-frame
///
/// This is defined as a newtype only to implement `TryFrom` on it.
#[derive(Debug, Clone)]
pub struct TaggedStringDataFrame(HashMap<String, OutputStringTimeSeries>);

impl TaggedStringDataFrame {
    /// Return an iterator over each named time-series
    pub fn iter(&self) -> impl Iterator<Item = (&String, &OutputStringTimeSeries)> {
        self.0.iter()
    }
}

#[allow(clippy::implicit_hasher)]
impl From<TaggedStringDataFrame> for HashMap<String, OutputStringTimeSeries> {
    fn from(dataframe: TaggedStringDataFrame) -> Self {
        dataframe.0
    }
}

/// Top-level response from InfluxDB
#[derive(Debug, Deserialize, Clone)]
pub struct InfluxDBResponse {
    /// Results for each query
    results: Vec<InfluxDBResult>,
}

impl TryFrom<&str> for InfluxDBResponse {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Ok(from_json_str(text)?)
    }
}

impl TryFrom<&[u8]> for InfluxDBResponse {
    type Error = SerdeJsonError;

    fn try_from(text: &[u8]) -> Result<Self, Self::Error> {
        from_json_slice(text)
    }
}

impl TryFrom<(&str, &InfluxDBResponse)> for TaggedDataFrame {
    type Error = Error;

    fn try_from((tag_name, result): (&str, &InfluxDBResponse)) -> Result<Self, Self::Error> {
        match *result.results.first().ok_or(Error::EmptyInfluxDBResults)? {
            InfluxDBResult::Error(ref result) => Err(Error::InfluxDBError(result.error.clone())),
            InfluxDBResult::Success(ref result) => {
                let seriess = result
                    .series
                    .iter()
                    .map(|series| {
                        if let &Series::TimeSeries(ref series) = series {
                            let values = series
                                .values
                                .iter()
                                .map(|&(ref instant, ref value)| {
                                    (*instant, value.as_f64().unwrap_or(f64::NAN))
                                })
                                .collect();
                            let tags = series.tags.as_ref().ok_or(Error::EmptyTags)?;
                            let tag_value = tags
                                .get(tag_name)
                                .ok_or(Error::MissingTag(tag_name.into()))?;
                            Ok((tag_value.clone(), values))
                        } else {
                            Err(Error::NotATimeSeries)
                        }
                    })
                    .collect::<Result<HashMap<String, OutputTimeSeries>, Self::Error>>()?;

                Ok(TaggedDataFrame(seriess))
            }
        }
    }
}

impl TryFrom<(&str, &InfluxDBResponse)> for TaggedStringDataFrame {
    type Error = Error;

    fn try_from((tag_name, result): (&str, &InfluxDBResponse)) -> Result<Self, Self::Error> {
        match *result.results.first().ok_or(Error::EmptyInfluxDBResults)? {
            InfluxDBResult::Error(ref result) => Err(Error::InfluxDBError(result.error.clone())),
            InfluxDBResult::Success(ref result) => {
                let seriess = result
                    .series
                    .iter()
                    .map(|series| {
                        if let &Series::TimeSeries(ref series) = series {
                            let values = series
                                .values
                                .iter()
                                .map(|&(ref instant, ref value)| {
                                    (*instant, value.as_str().unwrap_or("").to_owned())
                                })
                                .collect();
                            let tags = series.tags.as_ref().ok_or(Error::EmptyTags)?;
                            let tag_value = tags
                                .get(tag_name)
                                .ok_or(Error::MissingTag(tag_name.into()))?;
                            Ok((tag_value.clone(), values))
                        } else {
                            Err(Error::NotATimeSeries)
                        }
                    })
                    .collect::<Result<HashMap<String, OutputStringTimeSeries>, Self::Error>>()?;

                Ok(Self(seriess))
            }
        }
    }
}

#[allow(clippy::implicit_hasher)]
impl TryFrom<&InfluxDBResponse> for HashSet<String> {
    type Error = Error;

    fn try_from(result: &InfluxDBResponse) -> Result<Self, Self::Error> {
        match *result.results.first().ok_or(Error::EmptyInfluxDBResults)? {
            InfluxDBResult::Error(ref result) => Err(Error::InfluxDBError(result.error.clone())),
            InfluxDBResult::Success(ref result) => {
                let series: &Series = result.series.first().ok_or(Error::EmptySeries)?;
                if let &Series::TagSeries(ref series) = series {
                    let iterator: std::slice::Iter<(String, String)> = series.values.iter();
                    let values = iterator
                        .map(|&(ref _key, ref value)| value.clone())
                        .collect();
                    Ok(values)
                } else {
                    Err(Error::NotATagSeries)
                }
            }
        }
    }
}

/// An InfluxDB result
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum InfluxDBResult {
    /// An error result
    Error(InfluxDBErrorResult),

    /// A success result
    Success(InfluxDBSuccessResult),
}

/// Result for an InfluxDB query
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
struct InfluxDBSuccessResult {
    /// Statement ID
    statement_id: u32,

    /// Resulting series
    #[serde(default = "Vec::new")]
    series: Vec<Series>,
}

/// Result for an InfluxDB query
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
struct InfluxDBErrorResult {
    /// Statement ID
    statement_id: u32,

    /// Error message
    error: String,
}

/// A time-series within an InfluxDB result
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
struct TimeSeries {
    /// Series name
    name: String,

    /// Series columns
    columns: Vec<String>,

    /// Series values
    values: Vec<(DateTime<Utc>, JsonValue)>,

    /// Series tags
    tags: Option<HashMap<String, String>>,
}

/// A series of tags within an InfluxDB result
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
struct TagSeries {
    /// Series name
    name: String,

    /// Series columns
    columns: Vec<String>,

    /// Series values
    values: Vec<(String, String)>,

    /// Series tags
    tags: Option<HashMap<String, String>>,
}

/// A series within an InfluxDB result
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum Series {
    /// A time-series
    TimeSeries(TimeSeries),

    /// A series of tags
    TagSeries(TagSeries),
}
