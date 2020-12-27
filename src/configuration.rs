// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use serde::Deserialize;

use std::path::PathBuf;

use url::Url;


#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub style: StyleConfiguration,
    pub influxdb: InfluxdbConfiguration,
    pub charts: Vec<ChartConfiguration>,
    pub regions: Vec<GeographicalRegionConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct StyleConfiguration {
    pub font: String,
    pub palette: PaletteName,
    pub resolution: (u32, u32),
}

#[derive(Debug, Deserialize)]
pub enum PaletteName {
    Dark,
    Light,
}

#[derive(Debug, Deserialize)]
pub struct InfluxdbConfiguration {
    pub url: Url,
    pub database: String,
    pub username: String,
    pub password: String,
    pub cacert: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ChartConfiguration {
    Trend(TrendConfiguration),
    GeographicalMap(GeographicalMapConfiguration),
}


#[derive(Debug, Deserialize)]
pub struct TrendConfiguration {
    pub title: String,
    pub ylabel: Option<String>,
    pub xlabel_format: String,
    pub query: String,
    pub tag: String,
    pub tag_values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GeographicalMapConfiguration {
    pub title: String,
    pub unit: String,
    pub query: String,
    pub tag: String,
    pub bounds: (f64, f64),
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeographicalRegionConfiguration {
    pub name: String,
    pub coordinates: Vec<(f64, f64)>,
}
