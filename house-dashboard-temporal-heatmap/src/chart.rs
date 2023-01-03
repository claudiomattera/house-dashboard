// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for generating chart

use tracing::{debug, info};

use chrono::{DateTime, Duration, Local, Timelike, Utc};

use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    chart::{ChartBuilder, ChartContext},
    coord::{
        cartesian::Cartesian2d,
        types::{RangedCoordf64, RangedDateTime},
        Shift,
    },
    drawing::{DrawingArea, IntoDrawingArea},
    element::{Rectangle, Text},
    style::{
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont,
    },
};

use house_dashboard_common::{
    colormap::Colormap, configuration::StyleConfiguration, element::Colorbar, palette::SystemColor,
};

use crate::Error;
use crate::TemporalHeatMapConfiguration;

/// A chart context
type ChartContextAlias<'a, DB> =
    ChartContext<'a, DB, Cartesian2d<RangedDateTime<DateTime<Local>>, RangedCoordf64>>;

/// Draw an temporal heatmap chart
///
/// # Errors
///
/// Return and error when chart generation failed
pub fn draw_temporal_heatmap(
    temporal_heatmap: &TemporalHeatMapConfiguration,
    time_series: &[(DateTime<Utc>, f64)],
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error> {
    info!("Drawing temporal heatmap");

    let root = backend.into_drawing_area();
    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // Draw the title manually and create a new margin area
    draw_title(temporal_heatmap.title.as_str(), style, &root)?;
    let new_root = root.margin(30, 0, 0, 60);

    let time_series = convert_time_series_to_local_time(time_series);

    let mut chart = create_chart_context(temporal_heatmap.period.max_y(), &new_root, &time_series)?;

    draw_axes(temporal_heatmap, style, &mut chart)?;

    let colormap = Colormap::new_with_bounds_and_direction(
        temporal_heatmap.colormap.as_ref(),
        temporal_heatmap.bounds.0,
        temporal_heatmap.bounds.1,
        temporal_heatmap.reversed,
    );

    let fragments = create_fragments(temporal_heatmap, &colormap, &time_series);

    chart.draw_series(fragments)?;

    draw_colorbar(temporal_heatmap, style, colormap, &root)?;

    Ok(())
}

/// Draw title
fn draw_title<DB: DrawingBackend>(
    title: &str,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    let title_font = (style.font_name.as_str(), 16.0 * style.font_scale).into_font();
    let pos = Pos::new(HPos::Center, VPos::Top);

    let (width, _height) = root.dim_in_pixel();

    root.draw(&Text::new(
        title,
        (i32::try_from(width)? / 2, 10),
        title_font
            .color(&style.system_palette.pick(SystemColor::Foreground))
            .pos(pos),
    ))?;

    Ok(())
}

/// Create a chart context
fn create_chart_context<'a, DB: DrawingBackend + 'a>(
    max_y: f64,
    root: &'a DrawingArea<DB, Shift>,
    time_series: &[(DateTime<Local>, f64)],
) -> Result<ChartContextAlias<'a, DB>, Error> {
    debug!("Creating chart");

    let (min_x, max_x, min_y, max_y) = compute_range(max_y, time_series);

    let chart = ChartBuilder::on(root)
        .margin(5)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    Ok(chart)
}

/// Compute plot range
fn compute_range(
    max_y: f64,
    time_series: &[(DateTime<Local>, f64)],
) -> (DateTime<Local>, DateTime<Local>, f64, f64) {
    let mut min_x: DateTime<Local> = DateTime::<Utc>::MAX_UTC.with_timezone(&Local);
    let mut max_x: DateTime<Local> = DateTime::<Utc>::MIN_UTC.with_timezone(&Local);
    for &(date, _value) in time_series.iter() {
        min_x = min_x.min(date);
        max_x = max_x.max(date);
    }

    let min_y = 0.0;

    let padded_min_x = min_x
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.checked_sub_signed(Duration::hours(1)))
        .unwrap_or(min_x);
    let padded_max_x = max_x
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.checked_add_signed(Duration::hours(1)))
        .unwrap_or(max_x);

    debug!("Plot X range: [{}, {}]", padded_min_x, padded_max_x);
    debug!("Plot Y range: [{}, {}]", min_y, max_y);

    (padded_min_x, padded_max_x, min_y, max_y)
}

/// Draw chart axes
fn draw_axes<'a, DB: DrawingBackend + 'a>(
    temporal_heatmap: &TemporalHeatMapConfiguration,
    style: &'a StyleConfiguration,
    chart: &mut ChartContextAlias<'a, DB>,
) -> Result<(), Error> {
    debug!("Drawing axis");

    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));

    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(style.system_palette.pick(SystemColor::Foreground))
        .x_labels(3)
        .x_label_formatter(&|d| {
            d.format(temporal_heatmap.period.xlabel_format())
                .to_string()
        })
        .y_labels(4)
        .y_label_formatter(&|value| {
            format!("{0:.1$}", value, temporal_heatmap.precision.unwrap_or(0),)
        })
        .x_desc(temporal_heatmap.period.xlabel())
        .y_desc(temporal_heatmap.period.ylabel())
        .label_style(label_font.color(&style.system_palette.pick(SystemColor::Foreground)))
        .draw()?;

    Ok(())
}

/// Create chart fragments
fn create_fragments(
    temporal_heatmap: &TemporalHeatMapConfiguration,
    colormap: &Colormap,
    time_series: &[(DateTime<Local>, f64)],
) -> Vec<Rectangle<(DateTime<Local>, f64)>> {
    time_series
        .iter()
        .filter_map(|&(instant, value)| {
            temporal_heatmap
                .period
                .instant_to_rectangle(instant)
                .map(|((x1, x2), (y1, y2))| {
                    Rectangle::new(
                        [(x1, f64::from(y1)), (x2, f64::from(y2))],
                        colormap.get_color(value).filled(),
                    )
                })
        })
        .collect()
}

/// Draw a colorbar
fn draw_colorbar<DB>(
    temporal_heatmap: &TemporalHeatMapConfiguration,
    style: &StyleConfiguration,
    colormap: Colormap,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error>
where
    DB: DrawingBackend,
{
    debug!("Drawing colorbar");
    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale).into_font();

    let (width, height) = root.dim_in_pixel();

    let colorbar = Colorbar::new(
        (i32::try_from(width)? - 55, 40),
        (10, i32::try_from(height)? - 60),
        temporal_heatmap.bounds,
        temporal_heatmap.precision.unwrap_or(0),
        temporal_heatmap.unit.clone(),
        label_font,
        style.system_palette,
        colormap,
    );

    root.draw(&colorbar)?;

    Ok(())
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
