// Copyright Claudio Mattera 2023.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Gherkin behaviour-driven development tests

#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::ignored_unit_patterns,
    clippy::unwrap_used,
    clippy::needless_pass_by_value,
    clippy::panic_in_result_fn,
    clippy::unused_async
)]

use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Cursor};
use std::path::Path;

use plotters::backend::BitMapBackend;
use plotters::style::{register_font, FontStyle};

use chrono::{DateTime, Local, Utc};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use cucumber::{given, then, when, World};

use async_std::fs::read as read_file;
use async_std::fs::read_to_string as read_file_to_string;
use async_std::fs::write as write_file;

use serde_json::from_str as from_json_str;
use toml::from_str as from_toml_str;

use image::open as open_image;
use image::{ImageFormat, RgbImage};

use house_dashboard_common::configuration::StyleConfiguration;

use house_dashboard_geographical_heatmap::{
    draw_geographical_heatmap, GeographicalHeatMapConfiguration,
};
use house_dashboard_infrastructure_summary::{
    draw_infrastructure_summary, InfrastructureSummaryConfiguration,
};
use house_dashboard_proxmox_summary::{draw_proxmox_summary, ProxmoxSummaryConfiguration};
use house_dashboard_temporal_heatmap::{draw_temporal_heatmap, TemporalHeatMapConfiguration};
use house_dashboard_trend::{draw_trend, TrendConfiguration};

const TESTS_PATH: &str = "tests";
const DATA_PATH: &str = "tests/data";

fn main() {
    async_std::task::block_on(async {
        DashboardWorld::run(Path::new(TESTS_PATH).join("features/")).await;
    });
}

type TimeSeries = Vec<(DateTime<Utc>, f64)>;

#[derive(Debug, Default, World)]
struct DashboardWorld {
    x_range: Option<(DateTime<Local>, DateTime<Local>)>,
    style: Option<StyleConfiguration>,
    trend: Option<TrendConfiguration>,
    geographical_heatmap: Option<GeographicalHeatMapConfiguration>,
    temporal_heatmap: Option<TemporalHeatMapConfiguration>,
    infrastructure_summary: Option<InfrastructureSummaryConfiguration>,
    proxmox_summary: Option<ProxmoxSummaryConfiguration>,
    time_series_mapping: Option<HashMap<String, TimeSeries>>,
    time_series: Option<TimeSeries>,
    values_mapping: Option<HashMap<String, Option<f64>>>,
    hosts: Option<HashSet<String>>,
    statuses: Option<HashMap<String, String>>,
    loads: Option<HashMap<String, f64>>,
    now: Option<OffsetDateTime>,
    raw_image: Option<Vec<u8>>,
}

#[given(expr = "the time series mapping {string}")]
async fn given_time_series_mapping(
    world: &mut DashboardWorld,
    time_series_mapping_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let time_series_mapping_path = Path::new(DATA_PATH).join(time_series_mapping_filename);
    let time_series_mapping_content = read_file_to_string(time_series_mapping_path).await?;
    let time_series_mapping: HashMap<String, TimeSeries> =
        from_json_str(&time_series_mapping_content)?;
    world.time_series_mapping = Some(time_series_mapping);
    Ok(())
}

#[given(expr = "the values mapping {string}")]
async fn given_values_mapping(
    world: &mut DashboardWorld,
    values_mapping_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let values_mapping_path = Path::new(DATA_PATH).join(values_mapping_filename);
    let values_mapping_content = read_file_to_string(values_mapping_path).await?;
    let values_mapping: HashMap<String, Option<f64>> = from_json_str(&values_mapping_content)?;
    world.values_mapping = Some(values_mapping);
    Ok(())
}

#[given(expr = "the time series {string}")]
async fn given_time_series(
    world: &mut DashboardWorld,
    time_series_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let time_series_path = Path::new(DATA_PATH).join(time_series_filename);
    let time_series_content = read_file_to_string(time_series_path).await?;
    let time_series: TimeSeries = from_json_str(&time_series_content)?;
    world.time_series = Some(time_series);
    Ok(())
}

#[given(expr = "the hosts {string}")]
async fn given_hosts(
    world: &mut DashboardWorld,
    hosts_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let hosts_path = Path::new(DATA_PATH).join(hosts_filename);
    let hosts_content = read_file_to_string(hosts_path).await?;
    let hosts: HashSet<String> = from_json_str(&hosts_content)?;
    world.hosts = Some(hosts);
    Ok(())
}

#[given(expr = "the statuses {string}")]
async fn given_statuses(
    world: &mut DashboardWorld,
    statuses_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let statuses_path = Path::new(DATA_PATH).join(statuses_filename);
    let statuses_content = read_file_to_string(statuses_path).await?;
    let statuses: HashMap<String, String> = from_json_str(&statuses_content)?;
    world.statuses = Some(statuses);
    Ok(())
}

#[given(expr = "the loads {string}")]
async fn given_loads(
    world: &mut DashboardWorld,
    loads_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let loads_path = Path::new(DATA_PATH).join(loads_filename);
    let loads_content = read_file_to_string(loads_path).await?;
    let loads: HashMap<String, f64> = from_json_str(&loads_content)?;
    world.loads = Some(loads);
    Ok(())
}

#[given(expr = "the style configuration {string}")]
async fn given_style_configuration(
    world: &mut DashboardWorld,
    style_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_path = Path::new(DATA_PATH).join(style_filename);
    let style_content = read_file_to_string(&style_path).await?;
    let style: StyleConfiguration = from_toml_str(&style_content)?;

    load_font(
        &style.font_name,
        &style_path.parent().unwrap().join(&style.font_path),
    )
    .await?;

    world.style = Some(style);
    Ok(())
}

#[given(expr = "the data range {string} to {string}")]
fn given_x_range(
    world: &mut DashboardWorld,
    start: String,
    end: String,
) -> Result<(), Box<dyn std::error::Error>> {
    world.x_range = Some((
        start.parse::<DateTime<Local>>()?,
        end.parse::<DateTime<Local>>()?,
    ));
    Ok(())
}

#[given(expr = "the current time {string}")]
fn given_current_time(
    world: &mut DashboardWorld,
    now: String,
) -> Result<(), Box<dyn std::error::Error>> {
    world.now = Some(OffsetDateTime::parse(&now, &Rfc3339)?);
    Ok(())
}

#[given(expr = "the trend configuration {string}")]
async fn given_trend_configuration(
    world: &mut DashboardWorld,
    trend_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let trend_path = Path::new(DATA_PATH).join(trend_filename);
    let trend_content = read_file_to_string(trend_path).await?;
    let trend: TrendConfiguration = from_toml_str(&trend_content)?;
    world.trend = Some(trend);
    Ok(())
}

#[given(expr = "the temporal heatmap configuration {string}")]
async fn given_temporal_heatmap_configuration(
    world: &mut DashboardWorld,
    temporal_heatmap_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let temporal_heatmap_path = Path::new(DATA_PATH).join(temporal_heatmap_filename);
    let temporal_heatmap_content = read_file_to_string(temporal_heatmap_path).await?;
    let temporal_heatmap: TemporalHeatMapConfiguration = from_toml_str(&temporal_heatmap_content)?;
    world.temporal_heatmap = Some(temporal_heatmap);
    Ok(())
}

#[given(expr = "the geographical heatmap configuration {string}")]
async fn given_geographical_heatmap_configuration(
    world: &mut DashboardWorld,
    geographical_heatmap_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let geographical_heatmap_path = Path::new(DATA_PATH).join(geographical_heatmap_filename);
    let geographical_heatmap_content = read_file_to_string(geographical_heatmap_path).await?;
    let geographical_heatmap: GeographicalHeatMapConfiguration =
        from_toml_str(&geographical_heatmap_content)?;
    world.geographical_heatmap = Some(geographical_heatmap);
    Ok(())
}

#[given(expr = "the Proxmox summary configuration {string}")]
async fn given_proxmox_summary_configuration(
    world: &mut DashboardWorld,
    proxmox_summary_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let proxmox_summary_path = Path::new(DATA_PATH).join(proxmox_summary_filename);
    let proxmox_summary_content = read_file_to_string(proxmox_summary_path).await?;
    let proxmox_summary: ProxmoxSummaryConfiguration = from_toml_str(&proxmox_summary_content)?;
    world.proxmox_summary = Some(proxmox_summary);
    Ok(())
}

#[given(expr = "the infrastructure summary configuration {string}")]
async fn given_infrastructure_summary_configuration(
    world: &mut DashboardWorld,
    infrastructure_summary_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let infrastructure_summary_path = Path::new(DATA_PATH).join(infrastructure_summary_filename);
    let infrastructure_summary_content = read_file_to_string(infrastructure_summary_path).await?;
    let infrastructure_summary: InfrastructureSummaryConfiguration =
        from_toml_str(&infrastructure_summary_content)?;
    world.infrastructure_summary = Some(infrastructure_summary);
    Ok(())
}

#[when(expr = "drawing a trend chart")]
async fn when_drawing_trend_chart(
    world: &mut DashboardWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_configuration = world.style.as_ref().unwrap();
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);

    draw_trend(
        world.trend.as_ref().unwrap(),
        world.x_range.unwrap(),
        world.time_series_mapping.as_ref().unwrap(),
        world.style.as_ref().unwrap(),
        backend,
    )?;

    world.raw_image = Some(buffer);

    Ok(())
}

#[when(expr = "drawing a temporal heatmap chart")]
async fn when_drawing_temporal_heatmap_chart(
    world: &mut DashboardWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_configuration = world.style.as_ref().unwrap();
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);

    draw_temporal_heatmap(
        world.temporal_heatmap.as_ref().unwrap(),
        world.time_series.as_ref().unwrap(),
        world.style.as_ref().unwrap(),
        backend,
    )?;

    world.raw_image = Some(buffer);

    Ok(())
}

#[when(expr = "drawing a geographical heatmap chart")]
async fn when_drawing_geographical_heatmap_chart(
    world: &mut DashboardWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_configuration = world.style.as_ref().unwrap();
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);

    draw_geographical_heatmap(
        world.geographical_heatmap.as_ref().unwrap(),
        world.values_mapping.as_ref().unwrap(),
        world.style.as_ref().unwrap(),
        backend,
    )?;

    world.raw_image = Some(buffer);

    Ok(())
}

#[when(expr = "drawing a Proxmox summary chart")]
async fn when_drawing_proxmox_summary_chart(
    world: &mut DashboardWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_configuration = world.style.as_ref().unwrap();
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);

    draw_proxmox_summary(
        world.proxmox_summary.as_ref().unwrap(),
        world.hosts.as_ref().unwrap(),
        world.statuses.as_ref().unwrap(),
        world.loads.as_ref().unwrap(),
        world.style.as_ref().unwrap(),
        backend,
    )?;

    world.raw_image = Some(buffer);

    Ok(())
}

#[when(expr = "drawing an infrastructure summary chart")]
async fn when_drawing_infrastructure_summary_chart(
    world: &mut DashboardWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    let style_configuration = world.style.as_ref().unwrap();
    let area = style_configuration.resolution.0 * style_configuration.resolution.1;
    let area_in_bytes = area as usize * 3;
    let mut buffer: Vec<u8> = vec![0; area_in_bytes];
    let backend = BitMapBackend::with_buffer(&mut buffer, style_configuration.resolution);

    draw_infrastructure_summary(
        world.infrastructure_summary.as_ref().unwrap(),
        world.now.unwrap(),
        world.hosts.as_ref().unwrap(),
        world.loads.as_ref().unwrap(),
        world.style.as_ref().unwrap(),
        backend,
    )?;

    world.raw_image = Some(buffer);

    Ok(())
}

#[then(expr = "the bitmap is the same as {string}")]
fn then_bitmap_is_same_as(
    world: &mut DashboardWorld,
    expected_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected_path = Path::new(DATA_PATH).join(expected_filename);
    let expected_image = open_image(expected_path)?.into_rgb8();

    let (width, height) = world.style.as_ref().unwrap().resolution;

    let raw = world.raw_image.as_ref().unwrap().to_owned();
    let image = RgbImage::from_raw(width, height, raw).unwrap();

    assert_eq!(image, expected_image);

    Ok(())
}

#[then(expr = "the bitmap is saved to {string}")]
async fn then_bitmap_is_saved_to(
    world: &mut DashboardWorld,
    output_filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new(DATA_PATH).join(output_filename);

    let (width, height) = world.style.as_ref().unwrap().resolution;

    let raw = world.raw_image.as_ref().unwrap().to_owned();
    let image = RgbImage::from_raw(width, height, raw).unwrap();

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image.write_to(&mut buffer, ImageFormat::Bmp)?;

    let buffer = buffer.into_inner()?.into_inner();
    write_file(&output_path, &buffer).await?;

    Ok(())
}

/// Load custom font from a TTF or OTF file
async fn load_font(name: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let font_bytes = read_file(path).await?.into_boxed_slice();
    let font_bytes: &'static [u8] = Box::leak(font_bytes);
    register_font(name, FontStyle::Normal, font_bytes)
        .map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "font error"))?;
    Ok(())
}
