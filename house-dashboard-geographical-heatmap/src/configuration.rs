// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use serde::Deserialize;

use house_dashboard_common::colormap::ColormapType;
use house_dashboard_common::duration::Iso8601Duration;

/// Chart configuration for infrastructure summary charts
#[derive(Debug, Deserialize)]
pub struct GeographicalHeatMapConfiguration {
    /// Chart title
    pub title: String,

    /// Precision
    pub precision: Option<usize>,

    /// Unit
    pub unit: String,

    /// InfluxDB measurement
    pub measurement: String,

    /// InfluxDB field
    pub field: String,

    /// InfluxDB field scale
    pub scale: Option<f64>,

    /// InfluxDB tag name
    pub tag: String,

    /// Time of data from now
    pub how_long_ago: Iso8601Duration,

    /// Heatmap bounds
    pub bounds: (f64, f64),

    /// Colormap
    pub colormap: Option<ColormapType>,

    /// Setting to reverse colormap
    pub reversed: Option<bool>,

    /// Values of colored tags
    pub colored_tag_values: Option<Vec<String>>,

    /// Regions
    pub regions: Vec<GeographicalRegionConfiguration>,
}

/// Region configuration
#[derive(Clone, Debug, Deserialize)]
pub struct GeographicalRegionConfiguration {
    /// Region name
    pub name: String,

    /// Region coordinates
    pub coordinates: Vec<(f64, f64)>,
}
