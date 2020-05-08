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
    Trend (TrendConfiguration),
}


#[derive(Debug, Deserialize)]
pub struct TrendConfiguration {
    pub title: String,
    pub ylabel: Option<String>,
    pub xlabel_format: String,
    pub query: String,
    pub tag: String,
}
