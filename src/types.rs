// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use chrono::{DateTime, Utc};

use serde::Deserialize;

pub type Reading = (DateTime<Utc>, Value);
pub type TimeSeries = Vec<Reading>;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Value {
    Float(f64),
    String(String),
}

impl Value {
    pub fn to_f64(self) -> f64 {
        match self {
            Self::Float(value) => value,
            _ => unreachable!(),
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Self::String(value) => value,
            _ => unreachable!(),
        }
    }
}
