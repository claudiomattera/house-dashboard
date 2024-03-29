// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for handling errors

use enterpolation::linear::error::LinearError;

/// An error occurred while creating a colormap
pub type ColormapCreationError = LinearError;
