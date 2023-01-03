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

#[cfg(feature = "infrastructure-summary-chart")]
use house_dashboard_infrastructure_summary::{
    process_infrastructure_summary, InfrastructureSummaryConfiguration,
};

#[cfg(feature = "proxmox-summary-chart")]
use house_dashboard_proxmox_summary::{process_proxmox_summary, ProxmoxSummaryConfiguration};

#[cfg(feature = "trend-chart")]
use house_dashboard_trend::{process_trend, TrendConfiguration};

#[cfg(feature = "geographical-heatmap-chart")]
use house_dashboard_geographical_heatmap::{
    process_geographical_heatmap, GeographicalHeatMapConfiguration,
};

#[cfg(feature = "temporal-heatmap-chart")]
use house_dashboard_temporal_heatmap::{process_temporal_heatmap, TemporalHeatMapConfiguration};

#[cfg(feature = "image-chart")]
use house_dashboard_image::{process_image, ImageConfiguration};

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
    #[cfg(feature = "infrastructure-summary-chart")]
    /// Chart configuration for infrastructure summary
    InfrastructureSummary(Box<InfrastructureSummaryConfiguration>),

    #[cfg(feature = "proxmox-summary-chart")]
    /// Chart configuration for infrastructure summary
    ProxmoxSummary(Box<ProxmoxSummaryConfiguration>),

    #[cfg(feature = "trend-chart")]
    /// Chart configuration for trend
    Trend(Box<TrendConfiguration>),

    #[cfg(feature = "geographical-heatmap-chart")]
    /// Chart configuration for trend
    GeographicalHeatMap(Box<GeographicalHeatMapConfiguration>),

    #[cfg(feature = "temporal-heatmap-chart")]
    /// Chart configuration for trend
    TemporalHeatMap(Box<TemporalHeatMapConfiguration>),

    #[cfg(feature = "image-chart")]
    /// Chart configuration for trend
    Image(Box<ImageConfiguration>),
}

impl Chart {
    /// Process a chart
    pub async fn process(
        self,
        style: &StyleConfiguration,
        index: usize,
    ) -> Result<(usize, Vec<u8>), Report> {
        match self {
            #[cfg(feature = "infrastructure-summary-chart")]
            Self::InfrastructureSummary(configuration) => {
                let bytes = process_infrastructure_summary(&configuration, style, index)
                    .await
                    .wrap_err("cannot process infrastructure summary chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "proxmox-summary-chart")]
            Self::ProxmoxSummary(configuration) => {
                let bytes = process_proxmox_summary(&configuration, style, index)
                    .await
                    .wrap_err("cannot process proxmox summary chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "trend-chart")]
            Self::Trend(configuration) => {
                let bytes = process_trend(&configuration, style, index)
                    .await
                    .wrap_err("cannot process trend chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "geographical-heatmap-chart")]
            Self::GeographicalHeatMap(configuration) => {
                let bytes = process_geographical_heatmap(&configuration, style, index)
                    .await
                    .wrap_err("cannot process geographical heatmap chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "temporal-heatmap-chart")]
            Self::TemporalHeatMap(configuration) => {
                let bytes = process_temporal_heatmap(&configuration, style, index)
                    .await
                    .wrap_err("cannot process temporal heatmap chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "image-chart")]
            Self::Image(configuration) => {
                let bytes = process_image(&configuration, style, index)
                    .await
                    .wrap_err("cannot process image chart")?;
                Ok((index, bytes))
            }
        }
    }
}
