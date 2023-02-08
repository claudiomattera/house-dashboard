// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use serde::Deserialize;

use house_dashboard_common::duration::Iso8601Duration;

/// Chart configuration for infrastructure summary charts
#[derive(Debug, Deserialize)]
pub struct TrendConfiguration {
    /// Chart title
    pub title: String,

    /// Y label
    pub ylabel: Option<String>,

    /// Y unit
    pub yunit: Option<String>,

    /// X label format
    pub xlabel_format: String,

    /// Precision
    pub precision: Option<usize>,

    /// Setting to draw the last value
    pub draw_last_value: Option<bool>,

    /// Setting to hide the legend
    pub hide_legend: Option<bool>,

    /// Top padding
    pub top_padding: Option<f64>,

    /// Setting to draw the horizontal grid
    pub draw_horizontal_grid: Option<bool>,

    /// Maximal number of X ticks
    pub max_x_ticks: Option<usize>,

    /// Maximal number of Y ticks
    pub max_y_ticks: Option<usize>,

    /// InfluxDB database
    pub database: String,

    /// InfluxDB measurement
    pub measurement: String,

    /// InfluxDB field
    pub field: String,

    /// InfluxDB field scale
    pub scale: Option<f64>,

    /// InfluxDB aggregator
    pub aggregator: Option<String>,

    /// InfluxDB tag name
    pub tag: String,

    /// Time of data from now
    pub how_long_ago: Iso8601Duration,

    /// Data frequency
    pub how_often: Option<Iso8601Duration>,

    /// InfluxDB tag values
    pub tag_values: Option<Vec<String>>,
}
