// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for handling errors

use core::num::TryFromIntError;

use thiserror::Error as ThisError;

use miette::Diagnostic;

use plotters::drawing::DrawingAreaErrorKind;

use time_tz::system::Error as TimeTzError;

use time_fmt::format::FormatError as TimeFormatError;

/// An error occurred generating a chart
#[derive(ThisError, Debug, Diagnostic)]
pub enum Error {
    /// Error in chart backend
    #[error("backend error")]
    Backend,

    /// Chart backend is already in use
    #[error("backend already in use")]
    Sharing,

    /// Invalid chart layout
    #[error("invalid layout")]
    Layout,

    /// Font error
    #[error("font error")]
    Font,

    /// Integer conversion failed
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),

    /// Time formatting failed
    #[error(transparent)]
    TimeFormat(#[from] TimeFormatError),

    /// Timezone error
    #[error(transparent)]
    TimeTz(#[from] TimeTzError),
}

impl<T: std::error::Error + Send + Sync> From<DrawingAreaErrorKind<T>> for Error {
    fn from(error: DrawingAreaErrorKind<T>) -> Self {
        match error {
            DrawingAreaErrorKind::BackendError(_) => Self::Backend,
            DrawingAreaErrorKind::SharingError => Self::Sharing,
            DrawingAreaErrorKind::LayoutError => Self::Layout,
        }
    }
}
