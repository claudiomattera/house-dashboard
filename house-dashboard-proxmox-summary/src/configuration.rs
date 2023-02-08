// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use serde::Deserialize;

use house_dashboard_common::duration::Iso8601Duration;

/// Chart configuration for Proxmox summary charts
#[derive(Debug, Deserialize)]
pub struct ProxmoxSummaryConfiguration {
    /// Time of data from now
    pub how_long_ago: Iso8601Duration,

    /// Chart title
    pub title: String,

    /// Suffix to strip from hostnames
    pub suffix: Option<String>,

    /// Vertical space between hostnames
    pub vertical_step: Option<i32>,

    /// Proxmox node fully-qualified domain name
    pub node_fqdn: String,
}
