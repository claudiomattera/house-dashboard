// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for handling errors

use thiserror::Error as ThisError;

use miette::Diagnostic;

use plotters::drawing::DrawingAreaErrorKind;

#[derive(ThisError, Debug, Diagnostic)]
pub enum Error {
    #[error("backend error")]
    Backend,

    #[error("backend already in use")]
    Sharing,

    #[error("invalid layout")]
    Layout,
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
