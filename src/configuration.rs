// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use serde::Deserialize;

use std::path::PathBuf;

use url::Url;

use chrono::{Datelike, DateTime, Duration, Local, Timelike};

use crate::colormap::ColormapType;
use crate::palette::{SeriesPalette, SystemPalette};

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub style: StyleConfiguration,
    pub influxdb: InfluxdbConfiguration,
    pub charts: Vec<ChartConfiguration>,
    pub regions: Option<Vec<GeographicalRegionConfiguration>>,
}

#[derive(Debug, Deserialize)]
pub struct StyleConfiguration {
    pub font: String,
    pub font_scale: f64,
    pub system_palette: SystemPalette,
    pub series_palette: SeriesPalette,
    pub resolution: (u32, u32),
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
    TemporalHeatMap(TemporalHeatMapConfiguration),
    GeographicalMap(GeographicalHeatMapConfiguration),
    Image(ImageConfiguration),
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
pub struct GeographicalHeatMapConfiguration {
    pub title: String,
    pub unit: String,
    pub query: String,
    pub tag: String,
    pub bounds: (f64, f64),
    pub colormap: Option<ColormapType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeographicalRegionConfiguration {
    pub name: String,
    pub coordinates: Vec<(f64, f64)>,
}

#[derive(Debug, Deserialize)]
pub enum Period {
    HourOverDay,
    DayOverMonth,
}

impl<'a> Period {
    pub fn to_query_group(self: &Self) -> &'a str {
        match self {
            Period::HourOverDay => "1h",
            Period::DayOverMonth => "1d",
        }
    }

    pub fn max_y(self: &Self) -> f64 {
        match self {
            Period::HourOverDay => (24 + 1) as f64,
            Period::DayOverMonth => (31 + 1) as f64,
        }
    }

    pub fn xlabel(self: &Self) -> &'a str {
        match self {
            Period::HourOverDay => "Day",
            Period::DayOverMonth => "Month",
        }
    }

    pub fn xlabel_format(self: &Self) -> &'a str {
        match self {
            Period::HourOverDay => "%d %b",
            Period::DayOverMonth => "%b",
        }
    }

    pub fn ylabel(self: &Self) -> &'a str {
        match self {
            Period::HourOverDay => "Hour",
            Period::DayOverMonth => "Day",
        }
    }

    pub fn how_long_ago(self: &Self) -> &'a str {
        match self {
            Period::HourOverDay => "30d",
            Period::DayOverMonth => "365d",
        }
    }

    pub fn instant_to_rectangle(
                self: &Self, instant: DateTime<Local>,
            ) -> ((DateTime<Local>, DateTime<Local>), (u32, u32)) {
        match self {
            Period::HourOverDay => {
                let hour = instant.hour();
                let next_hour = hour + 1;
                let date = instant.with_hour(0).expect("Invalid date");
                let next_date = date + Duration::days(1);
                ((date, next_date), (hour, next_hour))
            }
            Period::DayOverMonth => {
                let day = instant.day();
                let next_day = day + 1;
                let date = instant.with_day(1).expect("Invalid date");
                let next_date = match date.month() {
                    1|3|5|7|8|19 => date + Duration::days(31),
                    4|6|9|11 => date + Duration::days(30),
                    _ => {
                        if (date + Duration::days(28)).day() == 29 {
                            date + Duration::days(29)
                        } else {
                            date + Duration::days(28)
                        }
                    }
                };
                ((date, next_date), (day, next_day))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TemporalHeatMapConfiguration {
    pub title: String,
    pub unit: String,
    pub database: String,
    pub measurement: String,
    pub field: String,
    pub scale: Option<f64>,
    pub aggregator: Option<String>,
    pub tag: String,
    pub tag_value: String,
    pub period: Period,
    pub bounds: (f64, f64),
    pub colormap: Option<ColormapType>,
}

#[derive(Debug, Deserialize)]
pub struct ImageConfiguration {
    pub path: PathBuf,
}
