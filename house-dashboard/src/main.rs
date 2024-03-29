// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main module

use house_dashboard::main as lib_main;

use miette::Report;

/// Wrapper for library main function
fn main() -> Result<(), Report> {
    async_std::task::block_on(async { lib_main().await })?;
    Ok(())
}
