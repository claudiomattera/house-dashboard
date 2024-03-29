// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for generating chart

use tracing::{debug, info, trace};

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use time::OffsetDateTime;

use time_tz::{system::get_timezone, OffsetDateTimeExt};

use time_fmt::format::format_offset_date_time;

use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    coord::Shift,
    drawing::{DrawingArea, IntoDrawingArea},
    element::{Circle, Rectangle, Text},
    style::{
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont, ShapeStyle,
    },
};

use house_dashboard_common::{
    colormap::{interpolate_colors, Colormap, ColormapType},
    configuration::StyleConfiguration,
    element::Loadbar,
    palette::SystemColor,
};

use crate::Error;
use crate::InfrastructureSummaryConfiguration;

/// X position of hostname
const HOST_X: i32 = 50;

/// X position of status
const STATUS_X: i32 = 220;

/// X position of load
const LOAD_X: i32 = 280;

/// Maximal load
const MAX_LOAD: f64 = 1.0;

/// Header height
const HEADER_HEIGHT: i32 = 32;

/// Draw an infrastructure summary chart
///
/// # Errors
///
/// Return and error when chart generation failed
pub fn draw_infrastructure_summary<S>(
    infrastructure_summary: &InfrastructureSummaryConfiguration,
    now: OffsetDateTime,
    hosts: &HashSet<String, S>,
    loads: &HashMap<String, f64, S>,
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error>
where
    S: BuildHasher,
{
    info!("Drawing infrastructure summary");

    let root = backend.into_drawing_area();
    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // Draw the title manually and create a new margin area
    let title_height = draw_title(infrastructure_summary.title.as_str(), style, &root)?;
    let new_root = root.margin(title_height, 0, 0, 0);

    draw_header(style, &new_root)?;

    let loads: Vec<(&String, Option<f64>)> = prepare_sorted_loads(hosts, loads);
    draw_hosts(
        loads.as_slice(),
        infrastructure_summary.vertical_step.unwrap_or(20),
        infrastructure_summary.suffix.as_ref(),
        style,
        &new_root,
    )?;

    if let Some(ref last_update_format) = infrastructure_summary.last_update_format {
        draw_footer(last_update_format, now, style, &new_root)?;
    }

    Ok(())
}

/// Prepare sorted pairs (host, load)
fn prepare_sorted_loads<'a, S>(
    hosts: &'a HashSet<String, S>,
    loads: &HashMap<String, f64, S>,
) -> Vec<(&'a String, Option<f64>)>
where
    S: BuildHasher,
{
    let mut hosts = hosts.iter().collect::<Vec<&String>>();
    hosts.sort();

    hosts
        .into_iter()
        .map(|host| (host, loads.get(host).copied()))
        .collect()
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

/// Draw header
fn draw_header<DB: DrawingBackend>(
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    let header_pos = Pos::new(HPos::Center, VPos::Top);
    let header_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(header_pos);

    let text = "HOST";
    let half_width: i32 = (i32::try_from(text.len())? * 8) / 2;
    root.draw(&Text::new(text, (HOST_X, 10), &header_font))?;
    root.draw(&Rectangle::new(
        [(HOST_X - half_width - 1, 17), (HOST_X + half_width - 2, 17)],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    let text = "STATUS";
    let half_width: i32 = (i32::try_from(text.len())? * 8) / 2;
    root.draw(&Text::new(text, (STATUS_X, 10), &header_font))?;
    root.draw(&Rectangle::new(
        [
            (STATUS_X - half_width - 1, 17),
            (STATUS_X + half_width - 2, 17),
        ],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    let text = "LOAD";
    let half_width: i32 = (i32::try_from(text.len())? * 8) / 2;
    root.draw(&Text::new(text, (LOAD_X, 10), &header_font))?;
    root.draw(&Rectangle::new(
        [(LOAD_X - half_width - 1, 17), (LOAD_X + half_width - 2, 17)],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    Ok(())
}

/// Draw hosts
fn draw_hosts<DB>(
    loads: &[(&String, Option<f64>)],
    vertical_step: i32,
    suffix: Option<&String>,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error>
where
    DB: DrawingBackend,
{
    let colormap = Colormap::new_with_bounds(Some(ColormapType::Status).as_ref(), 0.0, MAX_LOAD)?;

    for (i, &(host, load)) in (0..).zip(loads) {
        debug!(
            "Processing host {} ({}, relative load: {})",
            i + 1,
            host,
            load.map_or_else(|| "None".to_owned(), |l| l.to_string(),)
        );

        let centered_y = HEADER_HEIGHT + vertical_step * i;

        draw_host_name(host, suffix, centered_y, style, root)?;
        draw_host_status(&colormap, load, centered_y, style, root)?;
        draw_host_loadbar(&colormap, load, centered_y, style, root)?;
    }

    Ok(())
}

/// Draw host name
fn draw_host_name<DB: DrawingBackend>(
    host: &str,
    suffix: Option<&String>,
    centered_y: i32,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    trace!("Drawing hostname");

    let host_pos = Pos::new(HPos::Left, VPos::Center);
    let host_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(host_pos);

    let short_hostname = match suffix {
        Some(suffix) => host.strip_suffix(suffix).unwrap_or(host),
        None => host,
    };
    root.draw(&Text::new(short_hostname, (15, centered_y), &host_font))?;

    Ok(())
}

/// Draw host status
fn draw_host_status<DB: DrawingBackend>(
    colormap: &Colormap,
    load: Option<f64>,
    centered_y: i32,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    trace!("Drawing status");
    let color = if load.is_some() {
        colormap.get_color(0.0).to_rgba()
    } else {
        colormap.get_color(MAX_LOAD).to_rgba()
    };
    let shape_style: ShapeStyle = ShapeStyle {
        color,
        filled: true,
        stroke_width: 0,
    };
    let border_style: ShapeStyle = ShapeStyle {
        color: interpolate_colors(
            color,
            style.system_palette.pick(SystemColor::LightForeground),
        ),
        filled: false,
        stroke_width: 1,
    };
    root.draw(&Circle::new((STATUS_X, centered_y), 7, shape_style))?;
    root.draw(&Circle::new((STATUS_X, centered_y), 7, border_style))?;

    Ok(())
}

/// Draw host load bar
fn draw_host_loadbar<DB: DrawingBackend>(
    colormap: &Colormap,
    load: Option<f64>,
    centered_y: i32,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    trace!("Drawing loadbar");

    if let Some(load) = load {
        let loadbar = Loadbar::new(
            (LOAD_X, centered_y),
            (40, 10),
            MAX_LOAD,
            load,
            &style.system_palette,
            colormap,
        );
        root.draw(&loadbar)?;
    }

    Ok(())
}

/// Draw footer
fn draw_footer<DB: DrawingBackend>(
    last_update_format: &str,
    now: OffsetDateTime,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error> {
    let (width, height) = root.dim_in_pixel();

    let footer_pos = Pos::new(HPos::Right, VPos::Bottom);
    let footer_font = (style.font_name.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(footer_pos);

    let timezone = get_timezone()?;
    let now = now.to_timezone(timezone);

    root.draw(&Text::new(
        format_offset_date_time(last_update_format, now)?,
        (i32::try_from(width)? - 10, i32::try_from(height)?),
        &footer_font,
    ))?;

    Ok(())
}
