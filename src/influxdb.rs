// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use tracing::*;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};

use reqwest::Client;
use reqwest::header;
use reqwest::Certificate;

use serde::Deserialize;

use url::Url;

use anyhow::{Context, Result};

use crate::error::DashboardError;
use crate::types::TimeSeries;


#[derive(Debug)]
pub struct InfluxdbClient {
    http_client: Client,
    base_url: Url,
    database: String,
    username: String,
    password: String,
}

impl InfluxdbClient {
    pub fn new(
                base_url: Url,
                database: String,
                username: String,
                password: String,
                ca_cert: Option<PathBuf>,
                dangerously_accept_invalid_certs: bool,
            ) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));

        let mut client_builder = Client::builder()
                .default_headers(headers);

        if let Some(ca_cert) = ca_cert {
            debug!("Adding certificate authority {}", ca_cert.display());
            let certificate_data = std::fs::read(ca_cert)?;
            let certificate = Certificate::from_pem(&certificate_data)?;
            client_builder = client_builder.add_root_certificate(certificate)
        }
        if dangerously_accept_invalid_certs {
            warn!("Disabling SSL validation!");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
        let http_client = client_builder.build()?;

        Ok(Self {
            http_client,
            base_url,
            database,
            username,
            password,
        })
    }

    #[tracing::instrument(
        name = "Fetching tag values",
        skip(self, filter_tag_name, filter_tag_value),
    )]
    pub async fn fetch_tag_values(
                &self,
                database: &str,
                measurement: &str,
                key: &str,
                filter_tag_name: &str,
                filter_tag_value: &str,
            ) -> Result<HashSet<String>> {
        let query = format!(
            "SHOW TAG VALUES ON {database} FROM {measurement}
            WITH KEY = \"{key}\" WHERE \"{filter_tag_name}\" = '{filter_tag_value}'",
            database = database,
            measurement = measurement,
            key = key,
            filter_tag_name = filter_tag_name,
            filter_tag_value = filter_tag_value,
        );

        let raw = self.send_request(&query).await?;

        let p: InfluxdbTextualResults = serde_json::from_str(&raw)?;
        let result = p.results[0].clone();
        let series: Vec<TextualSeries> = result.series.ok_or(DashboardError::EmptyTimeSeries)?;
        let series = series.first().ok_or(DashboardError::EmptyTimeSeries)?;
        let values = &series.values;

        let mut tag_values = HashSet::new();
        for tag_pair in values {
            let tag_name = tag_pair.get(0).ok_or(DashboardError::EmptyTimeSeries)?;
            let tag_value = tag_pair.get(1).ok_or(DashboardError::EmptyTimeSeries)?;
            if tag_name != key {
                warn!("Unexpected tag {}", tag_name);
                continue;
            }
            tag_values.insert(tag_value.to_owned());
        }

        Ok(tag_values)
    }

    #[tracing::instrument(
        name = "Fetching timeseries by tag",
        skip(self, query),
    )]
    pub async fn fetch_timeseries_by_tag(
                &self,
                query: &str,
                tag_name: &str,
            ) -> Result<HashMap<String, TimeSeries>> {
        let raw = self.send_request(query).await?;

        let p: InfluxdbResults = serde_json::from_str(&raw)
            .context("Failed to parse JSON returned from InfluxDB")?;

        let result = p.results[0].clone();

        let mut time_seriess = HashMap::new();

        let series = result.series.ok_or(DashboardError::EmptyTimeSeries)?;

        debug!("Fetched {} time-series", series.len());

        for raw_series in series {

            let time_series: TimeSeries = raw_series.values
                .iter()
                .map(|(timestamp, value): &(i64, Option<f64>)| {
                    if let Some(value) = value {
                        let datetime: DateTime<Utc> = Utc.timestamp(*timestamp as i64, 0);
                        Some((datetime, *value))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect();

            let tag_value = &raw_series.tags.unwrap()[tag_name];
            debug!(
                "Fetched {count} readings from {start} to {end} for {tag_name}={tag_value}",
                count = time_series.len(),
                start = time_series[0].0,
                end = time_series[time_series.len() - 1].0,
                tag_name = tag_name,
                tag_value = tag_value,
            );

            time_seriess.insert(tag_value.to_string(), time_series);
        }

        Ok(time_seriess)
    }

    async fn send_request(
                &self,
                query: &str
            ) -> Result<String> {

        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("db", &self.database);
        params.insert("epoch", "s");

        let query_url = self.base_url.join("query")?;

        debug!("Sending query {} to {}", query, query_url);

        let response = self.http_client
            .post(query_url)
            .basic_auth(&self.username, Some(&self.password))
            .form(&params)
            .send()
            .await?;

        response.error_for_status_ref()?;

        let text = response.text().await?;

        Ok(text)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct InfluxdbResults {
    pub results: Vec<InfluxdbResult>,
}

#[derive(Debug, Deserialize, Clone)]
struct InfluxdbResult {
    pub statement_id: u32,
    pub series: Option<Vec<Series>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Series {
    pub name: String,
    pub columns: Vec<String>,
    pub values: Vec<(i64, Option<f64>)>,
    pub tags: Option<HashMap<String, String>>,
}


#[derive(Debug, Deserialize, Clone)]
struct InfluxdbTextualResults {
    pub results: Vec<InfluxdbTextualResult>,
}

#[derive(Debug, Deserialize, Clone)]
struct InfluxdbTextualResult {
    pub statement_id: u32,
    pub series: Option<Vec<TextualSeries>>,
}

#[derive(Debug, Deserialize, Clone)]
struct TextualSeries {
    pub name: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
}
