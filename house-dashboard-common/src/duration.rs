// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for parsing ISO8601 durations

use chrono::Duration;

use serde::de::Error;
use serde::{Deserialize, Deserializer};

use regex::Regex;

#[derive(Debug)]
pub struct Iso8601Duration {
    pub duration: Duration,
}

impl<'de> Deserialize<'de> for Iso8601Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let duration = string_to_duration(&string)
            .ok_or_else(|| D::Error::custom("Not a ISO8601 duration".to_owned()))?;
        Ok(Iso8601Duration { duration })
    }
}

fn string_to_duration(string: &str) -> Option<Duration> {
    let duration_regex = Regex::new(concat!(
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
    .unwrap();

    let mut duration = Duration::zero();

    match duration_regex.captures(string) {
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
        None => None,
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
        let expected = Some(Duration::days(365 + 30 * 4));
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
