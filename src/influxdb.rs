// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};

use reqwest::Client;
use reqwest::header;
use reqwest::Certificate;

use serde::Deserialize;

use url::Url;

use anyhow::Result;

use crate::error::DashboardError;
use crate::types::TimeSeries;


#[derive(Debug)]
pub struct InfluxdbClient {
    base_url: Url,
    database: String,
    username: String,
    password: String,
    ca_cert: Option<PathBuf>,
}

impl InfluxdbClient {
    pub fn new(
                base_url: Url,
                database: String,
                username: String,
                password: String,
                ca_cert: Option<PathBuf>
            ) -> Self {
        Self {
            base_url,
            database,
            username,
            password,
            ca_cert,
        }
    }

    pub async fn fetch_timeseries_by_tag(
                self: &Self,
                query: &str,
                tag_name: &str,
            ) -> Result<HashMap<String, TimeSeries>> {
        let raw = self.send_request(query).await?;

        let p: InfluxdbResults = serde_json::from_str(&raw)?;

        let result = p.results[0].clone();

        let mut time_seriess = HashMap::new();

        let series = result.series.ok_or(DashboardError::EmptyTimeSeries)?;

        debug!("Fetched {} time-series", series.len());

        for raw_series in series {

            let time_series: TimeSeries = raw_series.values
                .iter()
                .map(|vs| {
                    let datetime: DateTime<Utc> = Utc.timestamp(vs[0] as i64, 0);
                    let value = vs[1];
                    (datetime, value)
                }).collect();

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
                self: &Self,
                query: &str
            ) -> Result<String> {

        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));

        let mut client_builder = Client::builder()
                .default_headers(headers);

        match &self.ca_cert {
            Some(ca_cert) => {
                debug!("Adding certificate authority {}", ca_cert.display());
                let certificate_data = std::fs::read(ca_cert)?;
                let certificate = Certificate::from_pem(&certificate_data)?;
                client_builder = client_builder.add_root_certificate(certificate)
            }
            None => {}
        }
        let client = client_builder.build()?;

        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("db", &self.database);
        params.insert("epoch", "s");

        let query_url = self.base_url.join("query")?;

        debug!("Sending query {} to {}", query, query_url);

        let response = client
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
    pub values: Vec<Vec<f64>>,
    pub tags: Option<HashMap<String, String>>,
}
