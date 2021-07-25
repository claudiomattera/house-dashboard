// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use serde::{Deserialize, Deserializer};
use serde::de::Error;

use regex::Regex;

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
    pub draw_markers: Option<bool>,
    pub resolution: (u32, u32),
}

#[derive(Debug, Deserialize)]
pub struct InfluxdbConfiguration {
    pub url: Url,
    pub database: String,
    pub username: String,
    pub password: String,
    pub cacert: Option<PathBuf>,
    pub dangerously_accept_invalid_certs: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ChartConfiguration {
    Trend(TrendConfiguration),
    TemporalHeatMap(TemporalHeatMapConfiguration),
    GeographicalHeatMap(GeographicalHeatMapConfiguration),
    Image(ImageConfiguration),
    InfrastructureSummary(InfrastructureSummaryConfiguration),
}


#[derive(Debug, Deserialize)]
pub struct TrendConfiguration {
    pub title: String,
    pub ylabel: Option<String>,
    pub yunit: Option<String>,
    pub xlabel_format: String,
    pub precision: Option<usize>,
    pub draw_last_value: Option<bool>,
    pub hide_legend: Option<bool>,
    pub top_padding: Option<f64>,
    pub draw_horizontal_grid: Option<bool>,
    pub max_x_ticks: Option<usize>,
    pub max_y_ticks: Option<usize>,
    pub database: String,
    pub measurement: String,
    pub field: String,
    pub scale: Option<f64>,
    pub aggregator: Option<String>,
    pub tag: String,
    pub how_long_ago: Iso8601Duration,
    pub how_often: Option<Iso8601Duration>,
    pub tag_values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GeographicalHeatMapConfiguration {
    pub title: String,
    pub precision: Option<usize>,
    pub unit: String,
    pub database: String,
    pub measurement: String,
    pub field: String,
    pub scale: Option<f64>,
    pub tag: String,
    pub how_long_ago: Iso8601Duration,
    pub bounds: (f64, f64),
    pub colormap: Option<ColormapType>,
    pub reversed: Option<bool>,
    pub colored_tag_values: Option<Vec<String>>,
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
    pub fn to_query_group(&self) -> &'a str {
        match self {
            Period::HourOverDay => "1h",
            Period::DayOverMonth => "1d",
        }
    }

    pub fn max_y(&self) -> f64 {
        match self {
            Period::HourOverDay => (24 + 1) as f64,
            Period::DayOverMonth => (31 + 1) as f64,
        }
    }

    pub fn xlabel(&self) -> &'a str {
        match self {
            Period::HourOverDay => "Day",
            Period::DayOverMonth => "Month",
        }
    }

    pub fn xlabel_format(&self) -> &'a str {
        match self {
            Period::HourOverDay => "%d %b",
            Period::DayOverMonth => "%b",
        }
    }

    pub fn ylabel(&self) -> &'a str {
        match self {
            Period::HourOverDay => "Hour",
            Period::DayOverMonth => "Day",
        }
    }

    pub fn how_long_ago(&self) -> &'a str {
        match self {
            Period::HourOverDay => "30d",
            Period::DayOverMonth => "365d",
        }
    }

    pub fn instant_to_rectangle(
                &self, instant: DateTime<Local>,
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
    pub precision: Option<usize>,
    pub colormap: Option<ColormapType>,
}

#[derive(Debug, Deserialize)]
pub struct ImageConfiguration {
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Iso8601Duration {
    pub duration: Duration,
}

impl<'de> Deserialize<'de> for Iso8601Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {
            let string = String::deserialize(deserializer)?;
            let duration = string_to_duration(&string)
                .ok_or_else(|| D::Error::custom("Not a ISO8601 duration".to_owned()))?;
            Ok(Iso8601Duration{duration})
        }
}

fn string_to_duration(string: &str) -> Option<Duration> {
    let duration_regex = Regex::new(
        concat!(
            r"^P",
            r"((?P<years>\d+)Y)?",
            r"((?P<months>\d+)M)?",
            r"((?P<days>\d+)D)?",
            r"(T",
            r"((?P<hours>\d+)H)?",
            r"((?P<minutes>\d+)M)?",
            r"((?P<seconds>\d+)S)?",
            r")?",
            r"$",
        )
    ).unwrap();

    let mut duration = Duration::zero();

    match duration_regex.captures(&string) {
        Some(captures) => {
            if let Some(years_match) = captures.name("years") {
                let years: i64 = years_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::days(365 * years))?;
            }
            if let Some(months_match) = captures.name("months") {
                let months: i64 = months_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::days(30 * months))?;
            }
            if let Some(days_match) = captures.name("days") {
                let days: i64 = days_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::days(days))?;
            }
            if let Some(hours_match) = captures.name("hours") {
                let hours: i64 = hours_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::hours(hours))?;
            }
            if let Some(minutes_match) = captures.name("minutes") {
                let minutes: i64 = minutes_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::minutes(minutes))?;
            }
            if let Some(seconds_match) = captures.name("seconds") {
                let seconds: i64 = seconds_match.as_str().parse().ok()?;
                duration = duration.checked_add(&Duration::seconds(seconds))?;
            }
            Some(duration)
        }
        None => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_duration_years() {
        let string = "P1Y";
        let expected = Some(Duration::days(365));
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_years_months() {
        let string = "P1Y4M";
        let expected = Some(Duration::days(365 + 30*4));
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_years_days() {
        let string = "P1Y8D";
        let expected = Some(Duration::days(365 + 8));
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_years_days_hours() {
        let string = "P1Y8DT3H";
        let expected = Some(
                Duration::days(365 + 8)
                    .checked_add(&Duration::hours(3))
                    .unwrap(),
            );
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_years_days_hours_minutes() {
        let string = "P1Y8DT3H28M";
        let expected = Some(
                Duration::days(365 + 8)
                    .checked_add(&Duration::hours(3))
                    .unwrap()
                    .checked_add(&Duration::minutes(28))
                    .unwrap(),
            );
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_years_days_seconds() {
        let string = "P1Y8DT14S";
        let expected = Some(
                Duration::days(365 + 8)
                    .checked_add(&Duration::seconds(14))
                    .unwrap(),
            );
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_to_duration_hours_seconds() {
        let string = "PT7H14S";
        let expected = Some(
                Duration::hours(7)
                    .checked_add(&Duration::seconds(14))
                    .unwrap(),
            );
        let actual = string_to_duration(string);
        assert_eq!(actual, expected);
    }
}

#[derive(Debug, Deserialize)]
pub struct InfrastructureSummaryConfiguration {
    pub how_long_ago: Iso8601Duration,
    pub suffix: Option<String>,
    pub last_update_format: Option<String>,
    pub vertical_step: Option<i32>,
}
