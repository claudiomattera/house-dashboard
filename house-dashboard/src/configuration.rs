// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use std::path::PathBuf;

use miette::{Report, WrapErr};

use serde::Deserialize;

use url::Url;

use house_dashboard_common::configuration::StyleConfiguration;

use house_dashboard_infrastructure_summary::{
    process_infrastructure_summary, InfrastructureSummaryConfiguration,
};

/// InfluxDB configuration
#[derive(Debug, Deserialize)]
pub struct Influxdb {
    /// URL to InfluxDB instance
    pub url: Url,

    /// Database name
    pub database: String,

    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Path to custom certification authority certificate
    pub cacert: Option<PathBuf>,

    /// Set to true to accept invalid TLS certificates
    pub dangerously_accept_invalid_certs: Option<bool>,
}

/// Chart configuration
#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum Chart {
    /// Chart configuration for infrastructure summary
    InfrastructureSummary(InfrastructureSummaryConfiguration),
}

impl Chart {
    pub async fn process(self, style: &StyleConfiguration, index: usize) -> Result<(), Report> {
        match self {
            Self::InfrastructureSummary(configuration) => {
                process_infrastructure_summary(&configuration, &style, index)
                    .await
                    .wrap_err("cannot process infrastructure summary chart")
            }
        }
    }
}
