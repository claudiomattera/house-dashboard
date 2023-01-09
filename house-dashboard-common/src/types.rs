// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for common types

use chrono::{DateTime, Utc};

/// A time-series
pub type TimeSeries = Vec<(DateTime<Utc>, f64)>;
