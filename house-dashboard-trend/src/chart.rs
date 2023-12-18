// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for generating chart

use std::collections::HashMap;
use std::hash::BuildHasher;

use itertools::Itertools;

use tracing::{debug, info, warn};

use chrono::{DateTime, Local, Utc};

use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    chart::{ChartBuilder, ChartContext, SeriesLabelPosition},
    coord::{
        cartesian::Cartesian2d,
        types::{RangedCoordf64, RangedDateTime},
        Shift,
    },
    drawing::{DrawingArea, IntoDrawingArea},
    element::{Circle, PathElement, Text},
    series::LineSeries,
    style::{
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont, RGBAColor,
    },
};

use house_dashboard_common::{configuration::StyleConfiguration, palette::SystemColor};

use crate::Error;
use crate::TrendConfiguration;

/// A chart context
type ChartContextAlias<'a, DB> =
    ChartContext<'a, DB, Cartesian2d<RangedDateTime<DateTime<Local>>, RangedCoordf64>>;

/// Draw a trend chart
///
/// # Errors
///
/// Return and error when chart generation failed
pub fn draw_trend<S>(
    trend: &TrendConfiguration,
    x_range: (DateTime<Local>, DateTime<Local>),
    time_seriess: &HashMap<String, Vec<(DateTime<Utc>, f64)>, S>,
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error>
where
    S: BuildHasher + Default,
{
    info!("Drawing trend '{}'", trend.title.to_lowercase());

    let root = backend.into_drawing_area();
    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // Draw the title manually and create a new margin area
    let title_height = draw_title(trend.title.as_str(), style, &root)?;
    let new_root = root.margin(title_height, 0, 0, 0);

    let indices = compute_indices(trend.tag_values.as_ref(), time_seriess);

    let time_seriess = convert_time_seriess_to_local_time(time_seriess);

    let mut chart = create_chart_context(
        trend.top_padding,
        trend.min_y_range,
        x_range,
        &indices,
        &new_root,
        &time_seriess,
    )?;

    debug!("Drawing axis");
    draw_axes(trend, style, &mut chart)?;

    debug!("Plotting time-series");
    for (name, time_series) in time_seriess.iter().sorted_by_key(|pair| pair.0) {
        plot_time_series(
            trend,
            &indices,
            style,
            &new_root,
            &mut chart,
            name,
            time_series,
        )?;
    }

    if !trend.hide_legend.unwrap_or(false) {
        draw_legend(&mut chart, style)?;
    }

    Ok(())
}

/// Draw title
fn draw_title<DB: DrawingBackend>(
    title: &str,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<i32, Error> {
    let title_font = (style.font_name.as_str(), 16.0 * style.font_scale).into_font();
    let pos = Pos::new(HPos::Center, VPos::Top);

    let (width, _height) = root.dim_in_pixel();

    let (_box_width, box_height) = title_font.box_size(title).map_err(|_| Error::Font)?;
    let box_height = i32::try_from(box_height)?;
    let box_x = i32::try_from(width)? / 2;
    let box_y = box_height / 2;

    let vertical_skip = 5;

    root.draw(&Text::new(
        title,
        (box_x, box_y + vertical_skip),
        title_font
            .color(&style.system_palette.pick(SystemColor::Foreground))
            .pos(pos),
    ))?;

    Ok(box_height * 2)
}

/// Compute plots indices
fn compute_indices<S>(
    tag_values: Option<&Vec<String>>,
    time_seriess: &HashMap<String, Vec<(DateTime<Utc>, f64)>, S>,
) -> HashMap<String, usize>
where
    S: BuildHasher,
{
    let mut indices = HashMap::<String, usize>::new();
    match tag_values {
        Some(tag_values) => {
            for (index, name) in tag_values.iter().enumerate() {
                indices.insert(name.clone(), index);
            }
        }
        None => {
            for (index, (name, _)) in time_seriess.iter().enumerate() {
                indices.insert(name.clone(), index);
            }
        }
    };

    debug!("Indices: {:?}", indices);

    indices
}

/// Compute plot range
fn compute_range<S>(
    top_padding: Option<f64>,
    indices: &HashMap<String, usize>,
    time_seriess: &HashMap<String, Vec<(DateTime<Local>, f64)>, S>,
) -> (f64, f64)
where
    S: BuildHasher,
{
    let mut min_y = std::f64::MAX;
    let mut max_y = std::f64::MIN;

    for (name, time_series) in time_seriess {
        if !indices.contains_key(name) {
            debug!(
                "Skipping unexpected time-series '{}' for range computation",
                name,
            );
            continue;
        }
        for &(_date, value) in time_series {
            min_y = min_y.min(value);
            max_y = max_y.max(value);
        }
    }

    // Increase maximal Y range to make space for the legend
    let top_padding = top_padding.unwrap_or(0.0);
    max_y += top_padding * (max_y - min_y);

    debug!("Plot Y range: [{}, {}]", min_y, max_y);

    (min_y, max_y)
}

/// Convert time-series to local time
fn convert_time_seriess_to_local_time<S>(
    time_seriess: &HashMap<String, Vec<(DateTime<Utc>, f64)>, S>,
) -> HashMap<String, Vec<(DateTime<Local>, f64)>, S>
where
    S: BuildHasher + Default,
{
    time_seriess
        .iter()
        .map(|(name, time_series)| (name.clone(), convert_time_series_to_local_time(time_series)))
        .collect()
}

/// Convert time-series to local time
fn convert_time_series_to_local_time(
    time_series: &[(DateTime<Utc>, f64)],
) -> Vec<(DateTime<Local>, f64)> {
    time_series
        .iter()
        .map(|&(instant, value)| (instant.with_timezone(&Local), value))
        .collect()
}

/// Create a chart context
fn create_chart_context<'a, DB: DrawingBackend + 'a, S>(
    top_padding: Option<f64>,
    min_y_range: Option<f64>,
    (min_x, max_x): (DateTime<Local>, DateTime<Local>),
    indices: &HashMap<String, usize>,
    root: &'a DrawingArea<DB, Shift>,
    time_seriess: &HashMap<String, Vec<(DateTime<Local>, f64)>, S>,
) -> Result<ChartContextAlias<'a, DB>, Error>
where
    S: BuildHasher,
{
    debug!("Creating chart");

    let (mut min_y, mut max_y) = compute_range(top_padding, indices, time_seriess);

    if let Some(min_y_range) = min_y_range {
        let increment = min_y_range / 10.0;
        while max_y - min_y < min_y_range {
            min_y -= increment;
            max_y += increment;
        }
    }

    let chart = ChartBuilder::on(root)
        .margin(5)
        .x_label_area_size(20)
        .y_label_area_size(50)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    Ok(chart)
}

/// Plot a time-series
fn plot_time_series<'a, DB: DrawingBackend + 'a>(
    trend: &TrendConfiguration,
    indices: &HashMap<String, usize>,
    style: &'a StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
    chart: &mut ChartContextAlias<'a, DB>,
    name: &str,
    time_series: &[(DateTime<Local>, f64)],
) -> Result<(), Error> {
    let value_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(Pos::new(HPos::Right, VPos::Bottom));

    let index = if let Some(index) = indices.get(name) {
        *index
    } else {
        debug!("Unexpected tag value {}", name);
        return Ok(());
    };

    chart
        .draw_series(LineSeries::new(
            time_series.iter().map(|&(dt, value)| (dt, value)),
            style.series_palette.pick(index).stroke_width(3),
        ))?
        .label(name)
        .legend(move |(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                style.series_palette.pick(index).stroke_width(2),
            )
        });

    if style.draw_markers.unwrap_or(false) {
        chart.draw_series(time_series.iter().map(|&(dt, value)| {
            Circle::new((dt, value), 3, style.series_palette.pick(index).filled())
        }))?;
    }

    if trend.draw_last_value.unwrap_or(false) {
        if let Some(last_reading) = time_series.last() {
            let last_value = last_reading.1;
            let last_value_text = format!("{0:.1$}", last_value, trend.precision.unwrap_or(0),);

            let last_value_coordinates = chart.backend_coord(last_reading);

            root.draw(&Text::new(
                last_value_text,
                last_value_coordinates,
                &value_font,
            ))?;
        } else {
            warn!("Empty time-series '{}', cannot draw last value", name);
        }
    }

    Ok(())
}

/// Draw chart axes
fn draw_axes<'a, DB: DrawingBackend + 'a>(
    trend: &TrendConfiguration,
    style: &'a StyleConfiguration,
    chart: &mut ChartContextAlias<'a, DB>,
) -> Result<(), Error> {
    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));

    let ylabel = match (trend.ylabel.as_ref(), trend.yunit.as_ref()) {
        (Some(ylabel), Some(yunit)) => format!("{ylabel} [{yunit}]"),
        (Some(ylabel), None) => ylabel.clone(),
        (None, _) => String::new(),
    };

    let mut mesh = chart.configure_mesh();

    let mesh = if trend.draw_horizontal_grid.unwrap_or(false) {
        mesh.disable_x_mesh()
            .bold_line_style(style.system_palette.pick(SystemColor::Middle))
            .light_line_style(RGBAColor(0, 0, 0, 0.0))
    } else {
        mesh.disable_mesh()
    };

    mesh.axis_style(style.system_palette.pick(SystemColor::Foreground))
        .x_labels(trend.max_x_ticks.unwrap_or(4))
        .x_label_formatter(&|d| d.format(&trend.xlabel_format).to_string())
        .y_labels(trend.max_y_ticks.unwrap_or(5))
        .y_label_formatter(&|value| format!("{0:.1$}", value, trend.precision.unwrap_or(0)))
        .y_desc(ylabel)
        .label_style(label_font)
        .draw()?;

    Ok(())
}

/// Draw chart legend
fn draw_legend<'a, DB: DrawingBackend + 'a>(
    chart: &mut ChartContextAlias<'a, DB>,
    style: &StyleConfiguration,
) -> Result<(), Error> {
    debug!("Drawing legend");

    let legend_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));

    chart
        .configure_series_labels()
        .background_style(style.system_palette.pick(SystemColor::LightBackground))
        .border_style(style.system_palette.pick(SystemColor::LightForeground))
        .position(SeriesLabelPosition::UpperLeft)
        .label_font(legend_font)
        .draw()?;

    Ok(())
}
