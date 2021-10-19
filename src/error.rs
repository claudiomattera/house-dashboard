// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use thiserror::Error;

use image::error::ImageError;

use plotters::drawing::backend::DrawingErrorKind;
use plotters::drawing::DrawingAreaErrorKind;

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("Unknown error")]
    Unknown,
    #[error("Empty time-series")]
    EmptyTimeSeries,
    #[error("Unexpected tag value '{0}'")]
    UnexpectedTagValue(String),
    #[error("Non-existing tag value '{0}'")]
    NonexistingTagValue(String),
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
