// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use chrono::{DateTime, Utc};

pub type Reading = (DateTime<Utc>, f64);
pub type TimeSeries = Vec<Reading>;
