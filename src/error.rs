
use thiserror::Error;

use plotters::drawing::backend::DrawingErrorKind;
use plotters::drawing::DrawingAreaErrorKind;

#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("Unknown error")]
    Unknown,
    #[error("Unknown palette")]
    UnknownPalette,
    #[error("Empty time-series")]
    EmptyTimeSeries,
    #[error("Unexpected tag value \"{0}\"")]
    UnexpectedTagValue(String),
}

impl From<DrawingAreaErrorKind<DashboardError>> for DashboardError {
    fn from(error: DrawingAreaErrorKind<DashboardError>) -> Self {
       match error {
           DrawingAreaErrorKind::BackendError(error) => std::convert::From::from(error),
           _ => DashboardError::Unknown,
       }
    }
}

impl From<DrawingErrorKind<DashboardError>> for DashboardError {
    fn from(error: DrawingErrorKind<DashboardError>) -> Self {
       match error {
           DrawingErrorKind::DrawingError(error) => error,
           _ => DashboardError::Unknown,
       }
    }
}
