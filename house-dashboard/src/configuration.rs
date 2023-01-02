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

use house_dashboard_trend::{process_trend, TrendConfiguration};

use house_dashboard_geographical_heatmap::{
    process_geographical_heatmap, GeographicalHeatMapConfiguration,
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
    InfrastructureSummary(Box<InfrastructureSummaryConfiguration>),

    /// Chart configuration for trend
    Trend(Box<TrendConfiguration>),

    /// Chart configuration for trend
    GeographicalHeatMap(Box<GeographicalHeatMapConfiguration>),
}

impl Chart {
    /// Process a chart
    pub async fn process(
        self,
        style: &StyleConfiguration,
        index: usize,
    ) -> Result<(usize, Vec<u8>), Report> {
        match self {
            Self::InfrastructureSummary(configuration) => {
                let bytes = process_infrastructure_summary(&configuration, style, index)
                    .await
                    .wrap_err("cannot process infrastructure summary chart")?;
                Ok((index, bytes))
            }
            Self::Trend(configuration) => {
                let bytes = process_trend(&configuration, style, index)
                    .await
                    .wrap_err("cannot process trend chart")?;
                Ok((index, bytes))
            }
            Self::GeographicalHeatMap(configuration) => {
                let bytes = process_geographical_heatmap(&configuration, style, index)
                    .await
                    .wrap_err("cannot process geographical heatmap chart")?;
                Ok((index, bytes))
            }
        }
    }
}
