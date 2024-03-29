// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for parsing ISO8601 durations

use std::fmt::Error as FmtError;
use std::fmt::Write;

use time::Duration;

use serde::de::Error;
use serde::{Deserialize, Deserializer};

use regex::Regex;

/// A duration
#[derive(Debug)]
pub struct Iso8601Duration {
    /// Duration
    pub duration: Duration,
}

impl<'de> Deserialize<'de> for Iso8601Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let duration = parse_iso8601_duration(&string)
            .ok_or_else(|| D::Error::custom("Not a ISO8601 duration".to_owned()))?;
        Ok(Iso8601Duration { duration })
    }
}

/// Parse a duration from an ISO8601 formatted string
fn parse_iso8601_duration(string: &str) -> Option<Duration> {
    Regex::new(concat!(
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
    ))
    .ok()
    .and_then(|duration_regex| {
        let mut duration = Duration::ZERO;

        match duration_regex.captures(string) {
            Some(captures) => {
                if let Some(years_match) = captures.name("years") {
                    let years: i64 = years_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::days(365 * years))?;
                }
                if let Some(months_match) = captures.name("months") {
                    let months: i64 = months_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::days(30 * months))?;
                }
                if let Some(days_match) = captures.name("days") {
                    let days: i64 = days_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::days(days))?;
                }
                if let Some(hours_match) = captures.name("hours") {
                    let hours: i64 = hours_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::hours(hours))?;
                }
                if let Some(minutes_match) = captures.name("minutes") {
                    let minutes: i64 = minutes_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::minutes(minutes))?;
                }
                if let Some(seconds_match) = captures.name("seconds") {
                    let seconds: i64 = seconds_match.as_str().parse().ok()?;
                    duration = duration.checked_add(Duration::seconds(seconds))?;
                }
                Some(duration)
            }
            None => None,
        }
    })
}

/// Convert a duration to a duration string
///
/// # Errors
///
/// Return an error if formatting to a string fails.
pub fn duration_to_query(duration: &Duration) -> Result<String, FmtError> {
    let mut string = String::new();

    let seconds = duration.whole_seconds();
    if seconds > 0 {
        write!(&mut string, "{seconds}s")?;
    }

    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iso8601_duration_years() {
        let string = "P1Y";
        let expected = Some(Duration::days(365));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_years_months() {
        let string = "P1Y4M";
        let expected = Some(Duration::days(365 + 30 * 4));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_years_days() {
        let string = "P1Y8D";
        let expected = Some(Duration::days(365 + 8));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_years_days_hours() {
        let string = "P1Y8DT3H";
        let expected = Some(Duration::hours(8955));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_years_days_hours_minutes() {
        let string = "P1Y8DT3H28M";
        let expected = Some(Duration::minutes(537_328));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_years_days_seconds() {
        let string = "P1Y8DT14S";
        let expected = Some(Duration::seconds(32_227_214));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_iso8601_duration_hours_seconds() {
        let string = "PT7H14S";
        let expected = Some(Duration::seconds(25_214));
        let actual = parse_iso8601_duration(string);
        assert_eq!(actual, expected);
    }
}
