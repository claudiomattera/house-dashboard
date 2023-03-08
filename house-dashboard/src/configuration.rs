// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data structures for parsing configuration

use std::path::PathBuf;
use std::time::Duration;

use async_std::task::sleep;

use tracing::{info, warn};

use miette::miette;
use miette::{Report, WrapErr};

use serde::Deserialize;

use url::Url;

use house_dashboard_common::configuration::StyleConfiguration;

use house_dashboard_influxdb::InfluxDBClient;

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

    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Path to custom certification authority certificate
    pub cacert: Option<PathBuf>,

    /// Set to true to accept invalid TLS certificates
    pub dangerously_accept_invalid_certs: Option<bool>,
}

/// Maximum attempts for processing the chart
const MAX_ATTEMPTS: u32 = 4;

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
    ///
    /// # Errors
    ///
    /// Return an error if the operation fails more than [`MAX_ATTEMPTS`] times
    /// in a row.
    pub async fn process(
        self,
        influxdb_client: InfluxDBClient,
        style: &StyleConfiguration,
        index: usize,
    ) -> Result<(usize, Vec<u8>), Report> {
        for attempt in 0..MAX_ATTEMPTS {
            match self
                .process_inner(influxdb_client.clone(), style, index)
                .await
            {
                ok @ Ok(_) => return ok,
                Err(error) => {
                    warn!("Attempt {} failed: {:?}", attempt + 1, error);

                    let delay = Duration::from_secs(2_u64.checked_pow(attempt + 3).unwrap_or(10));
                    info!("Trying again in {}s", delay.as_secs());
                    sleep(delay).await;
                    continue;
                }
            }
        }

        Err(miette!("Operation failed more than {} times", MAX_ATTEMPTS))
    }

    /// Process a chart
    async fn process_inner(
        &self,
        influxdb_client: InfluxDBClient,
        style: &StyleConfiguration,
        index: usize,
    ) -> Result<(usize, Vec<u8>), Report> {
        match self {
            #[cfg(feature = "infrastructure-summary-chart")]
            Self::InfrastructureSummary(ref configuration) => {
                let bytes =
                    process_infrastructure_summary(&influxdb_client, configuration, style, index)
                        .await
                        .wrap_err("cannot process infrastructure summary chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "proxmox-summary-chart")]
            Self::ProxmoxSummary(ref configuration) => {
                let bytes = process_proxmox_summary(&influxdb_client, configuration, style, index)
                    .await
                    .wrap_err("cannot process proxmox summary chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "trend-chart")]
            Self::Trend(ref configuration) => {
                let bytes = process_trend(&influxdb_client, configuration, style, index)
                    .await
                    .wrap_err("cannot process trend chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "geographical-heatmap-chart")]
            Self::GeographicalHeatMap(ref configuration) => {
                let bytes =
                    process_geographical_heatmap(&influxdb_client, configuration, style, index)
                        .await
                        .wrap_err("cannot process geographical heatmap chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "temporal-heatmap-chart")]
            Self::TemporalHeatMap(ref configuration) => {
                let bytes = process_temporal_heatmap(&influxdb_client, configuration, style, index)
                    .await
                    .wrap_err("cannot process temporal heatmap chart")?;
                Ok((index, bytes))
            }

            #[cfg(feature = "image-chart")]
            Self::Image(ref configuration) => {
                let bytes = process_image(configuration, style, index)
                    .await
                    .wrap_err("cannot process image chart")?;
                Ok((index, bytes))
            }
        }
    }
}
