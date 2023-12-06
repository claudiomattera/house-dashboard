// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for handling errors

use thiserror::Error;

use image::error::ImageError;

use plotters::drawing::backend::DrawingErrorKind;
use plotters::drawing::DrawingAreaErrorKind;

/// An error occurred generating a dashboard
#[derive(Error, Debug)]
pub enum DashboardError {
    /// An unknown error
    #[error("Unknown error")]
    Unknown,

    /// InfluxDB server returned an empty time-series
    #[error("Empty time-series")]
    EmptyTimeSeries,

    /// InfluxDB server returned an unexpected tag value
    #[error("Unexpected tag value '{0}'")]
    UnexpectedTagValue(String),

    /// InfluxDB server returned a non-existing tag value
    #[error("Non-existing tag value '{0}'")]
    NonexistingTagValue(String),

    /// Image generation failed
    #[error("Image error '{0}'")]
    ImageError(ImageError),
}

impl<T: std::error::Error + Send + Sync> From<DrawingAreaErrorKind<T>> for DashboardError {
    fn from(_error: DrawingAreaErrorKind<T>) -> Self {
        DashboardError::Unknown
    }
}

impl<T: std::error::Error + Send + Sync> From<DrawingErrorKind<T>> for DashboardError {
    fn from(_error: DrawingErrorKind<T>) -> Self {
        DashboardError::Unknown
    }
}
