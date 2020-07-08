
use thiserror::Error;

use image::error::ImageError;

use plotters::drawing::DrawingAreaErrorKind;
use plotters::drawing::backend::DrawingErrorKind;

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("Unknown error")]
    Unknown,
    #[error("Empty time-series")]
    EmptyTimeSeries,
    #[error("Unexpected tag value \"{0}\"")]
    UnexpectedTagValue(String),
    #[error("Non-existing tag value \"{0}\"")]
    NonexistingTagValue(String),
    #[error("Image error \"{0}\"")]
    ImageError(ImageError),
}

impl <T: std::error::Error + Send + Sync> From<DrawingAreaErrorKind<T>> for DashboardError {
    fn from(_error: DrawingAreaErrorKind<T>) -> Self {
        DashboardError::Unknown
    }
}

impl <T: std::error::Error + Send + Sync> From<DrawingErrorKind<T>> for DashboardError {
    fn from(_error: DrawingErrorKind<T>) -> Self {
        DashboardError::Unknown
    }
}
