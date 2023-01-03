// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use serde::Deserialize;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};

use house_dashboard_common::colormap::ColormapType;

/// Chart configuration for temporal heatmap charts
#[derive(Debug, Deserialize)]
pub struct TemporalHeatMapConfiguration {
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

    /// InfluxDB aggregator
    pub aggregator: Option<String>,

    /// InfluxDB field scale
    pub scale: Option<f64>,

    /// InfluxDB tag name
    pub tag: String,

    /// InfluxDB tag value
    pub tag_value: String,

    /// Period
    pub period: Period,

    /// Heatmap bounds
    pub bounds: (f64, f64),

    /// Colormap
    pub colormap: Option<ColormapType>,

    /// Setting to reverse colormap
    pub reversed: Option<bool>,
}

/// Period
#[derive(Debug, Deserialize)]
pub enum Period {
    /// Hour over day
    HourOverDay,

    /// Day over month
    DayOverMonth,
}

/// A rectangle whose X values are on time domain
type TemporalRectangle = ((DateTime<Local>, DateTime<Local>), (u32, u32));

impl<'a> Period {
    /// Map period to InfluxDB query group
    pub fn to_query_group(&self) -> &'a str {
        match *self {
            Period::HourOverDay => "1h",
            Period::DayOverMonth => "1d",
        }
    }

    /// Get period max Y
    pub fn max_y(&self) -> f64 {
        match *self {
            Period::HourOverDay => f64::from(24 + 1),
            Period::DayOverMonth => f64::from(31 + 1),
        }
    }

    /// Get period X label
    pub fn xlabel(&self) -> &'a str {
        match *self {
            Period::HourOverDay => "Day",
            Period::DayOverMonth => "Month",
        }
    }

    /// Get period X label format
    pub fn xlabel_format(&self) -> &'a str {
        match *self {
            Period::HourOverDay => "%d %b",
            Period::DayOverMonth => "%b",
        }
    }

    /// Get period Y label
    pub fn ylabel(&self) -> &'a str {
        match *self {
            Period::HourOverDay => "Hour",
            Period::DayOverMonth => "Day",
        }
    }

    /// Get period time from present
    pub fn how_long_ago(&self) -> &'a str {
        match *self {
            Period::HourOverDay => "30d",
            Period::DayOverMonth => "365d",
        }
    }

    /// Map period to a rectangle
    pub fn instant_to_rectangle(&self, instant: DateTime<Local>) -> Option<TemporalRectangle> {
        match *self {
            Period::HourOverDay => {
                let hour = instant.hour();
                let next_hour = hour + 1;
                if let Some(date) = instant.with_hour(0) {
                    let next_date = date + Duration::days(1);
                    Some(((date, next_date), (hour, next_hour)))
                } else {
                    None
                }
            }
            Period::DayOverMonth => {
                let day = instant.day();
                let next_day = day + 1;
                if let Some(date) = instant.with_day(1) {
                    let next_date = match date.month() {
                        1 | 3 | 5 | 7 | 8 | 19 => date + Duration::days(31),
                        4 | 6 | 9 | 11 => date + Duration::days(30),
                        _ => {
                            if (date + Duration::days(28)).day() == 29 {
                                date + Duration::days(29)
                            } else {
                                date + Duration::days(28)
                            }
                        }
                    };
                    Some(((date, next_date), (day, next_day)))
                } else {
                    None
                }
            }
        }
    }
}
