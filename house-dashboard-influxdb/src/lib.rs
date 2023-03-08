// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types and functions for fetching data from InfluxDB

#![deny(
    missing_docs,
    clippy::cargo,
    clippy::pedantic,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
#![deny(
    clippy::allow_attributes_without_reason,
    clippy::clone_on_ref_ptr,
    clippy::else_if_without_else,
    clippy::expect_used,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::str_to_string,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used,
    clippy::verbose_file_reads
)]

use std::collections::{HashMap, HashSet};

use async_std::io::ReadExt as _;

use async_std::sync::{Arc, Mutex};

use tracing::{debug, trace};

use url::Url;

use chrono::{DateTime, Utc};

use isahc::{HttpClient, Request};

use serde_html_form::to_string as to_form_urlencoded;

mod error;
pub use error::Error;

mod influxql;
use influxql::{InfluxDBResponse, TaggedDataFrame, TaggedStringDataFrame};

/// A client to InfluxDB
#[derive(Clone, Debug)]
pub struct InfluxDBClient {
    /// The InfluxDB server base URL
    base_url: Url,

    /// The underlying HTTP client
    http_client: Arc<Mutex<HttpClient>>,
}

impl InfluxDBClient {
    /// Create a new InfluxDB client from an HTTP client
    #[must_use]
    pub fn new(base_url: Url, http_client: HttpClient) -> Self {
        let http_client = Arc::new(Mutex::new(http_client));
        Self {
            base_url,
            http_client,
        }
    }

    /// Fetch existing tag values for a given key in a measurement
    ///
    /// # Errors
    ///
    /// Return an error when the HTTP connection fails, when the InfluxDB
    /// connection fails, or when the response cannot be parsed.
    pub async fn fetch_tag_values(
        &self,
        database: &str,
        measurement: &str,
        key: &str,
        filter_tag_name: &str,
        filter_tag_value: &str,
    ) -> Result<HashSet<String>, Error> {
        let query = format!(
            r#"SHOW TAG VALUES ON "{database}" FROM {measurement} WITH KEY = "{key}" WHERE "{filter_tag_name}" = '{filter_tag_value}'"#
        );
        let results = self.request(&query).await?;
        let tags: HashSet<String> = (&results).try_into()?;
        debug!("Fetched {} tags", tags.len());
        Ok(tags)
    }

    /// Fetch a list of named time-series, one per each tag value
    ///
    /// # Errors
    ///
    /// Return an error when the HTTP connection fails, when the InfluxDB
    /// connection fails, or when the response cannot be parsed.
    pub async fn fetch_tagged_dataframes(
        &self,
        query: &str,
        tag_name: &str,
    ) -> Result<HashMap<String, Vec<(DateTime<Utc>, f64)>>, Error> {
        let results = self.request(query).await?;

        let a: (&str, &InfluxDBResponse) = (tag_name, &results);
        let dataframe: TaggedDataFrame = a.try_into()?;
        let seriess: HashMap<String, Vec<(DateTime<Utc>, f64)>> = dataframe.into();

        debug!("Fetched {} time-series", seriess.len());

        Ok(seriess)
    }

    /// Fetch a list of named string time-series, one per each tag value
    ///
    /// # Errors
    ///
    /// Return an error when the HTTP connection fails, when the InfluxDB
    /// connection fails, or when the response cannot be parsed.
    pub async fn fetch_tagged_string_dataframes(
        &self,
        query: &str,
        tag_name: &str,
    ) -> Result<HashMap<String, Vec<(DateTime<Utc>, String)>>, Error> {
        let results = self.request(query).await?;

        let a: (&str, &InfluxDBResponse) = (tag_name, &results);
        let dataframe: TaggedStringDataFrame = a.try_into()?;
        let seriess: HashMap<String, Vec<(DateTime<Utc>, String)>> = dataframe.into();

        debug!("Fetched {} time-series", seriess.len());

        Ok(seriess)
    }

    /// Send a request to InfluxDB server and parse its response
    async fn request(&self, query: &str) -> Result<InfluxDBResponse, Error> {
        let params = [("q", query)];
        let body = to_form_urlencoded(params)?;

        trace!("Request body: {}", body);

        let url = self.base_url.join("/query")?;

        let request = Request::post(url.to_string())
            .header("Accept", "application/json")
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)?;

        let http_client = self.http_client.lock().await;

        let mut response = http_client.send_async(request).await?;

        debug!("Response status: {:?}", response.status());

        let body = response.body_mut();

        let mut buffer = Vec::new();
        body.read_to_end(&mut buffer).await?;

        trace!("Response text: {}", String::from_utf8_lossy(&buffer));

        let influxdb_results: InfluxDBResponse = buffer.as_slice().try_into()?;

        trace!("Parsed response: {:?}", influxdb_results);

        Ok(influxdb_results)
    }
}
