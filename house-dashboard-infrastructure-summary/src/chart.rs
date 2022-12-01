// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use tracing::*;

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local, Utc};

use plotters::{
    backend::BitMapBackend,
    drawing::IntoDrawingArea,
    element::{Circle, Rectangle, Text},
    style::{
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont, ShapeStyle,
    },
};

use house_dashboard_common::{
    colormap::{Colormap, ColormapType},
    configuration::StyleConfiguration,
    palette::SystemColor,
};

use crate::Error;
use crate::InfrastructureSummaryConfiguration;
use crate::Loadbar;

pub fn draw_infrastructure_summary(
    infrastructure_summary: &InfrastructureSummaryConfiguration,
    now: DateTime<Utc>,
    hosts: HashSet<String>,
    loads: HashMap<String, f64>,
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error> {
    info!("Drawing infrastructure summary");

    let title_font = (style.font_name.as_str(), 16.0 * style.font_scale).into_font();
    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale).into_font();

    let root = backend.into_drawing_area();
    let (width, height) = root.dim_in_pixel();

    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // In order to make room for the colorbar, we need to set `margin_right()`
    // but that would make the title not centred.
    // So we must draw the title manually, and also create a new margin area.
    let pos = Pos::new(HPos::Center, VPos::Top);
    root.draw(&Text::new(
        "INFRASTRUCTURE",
        (width as i32 / 2, 10),
        title_font
            .color(&style.system_palette.pick(SystemColor::Foreground))
            .pos(pos),
    ))?;

    let new_root = root.margin(30, 0, 0, 0);
    let (_new_width, _new_height) = new_root.dim_in_pixel();

    let mut hosts = hosts.iter().collect::<Vec<&String>>();
    hosts.sort();

    let header_pos = Pos::new(HPos::Center, VPos::Top);
    let header_font = label_font
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(header_pos);

    let host_pos = Pos::new(HPos::Left, VPos::Center);
    let host_font = label_font
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(host_pos);

    let footer_pos = Pos::new(HPos::Right, VPos::Bottom);
    let footer_font = label_font
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(footer_pos);

    const HOST_X: i32 = 50;
    const STATUS_X: i32 = 220;
    const LOAD_X: i32 = 280;

    let text = "HOST";
    let half_width: i32 = (text.len() as i32 * 8) / 2;
    new_root.draw(&Text::new(text, (HOST_X, 10), &header_font))?;
    new_root.draw(&Rectangle::new(
        [(HOST_X - half_width, 19), (HOST_X + half_width, 19)],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    let text = "STATUS";
    let half_width: i32 = (text.len() as i32 * 8) / 2;
    new_root.draw(&Text::new(text, (STATUS_X, 10), &header_font))?;
    new_root.draw(&Rectangle::new(
        [(STATUS_X - half_width, 19), (STATUS_X + half_width, 19)],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    let text = "LOAD";
    let half_width: i32 = (text.len() as i32 * 8) / 2;
    new_root.draw(&Text::new(text, (LOAD_X, 10), &header_font))?;
    new_root.draw(&Rectangle::new(
        [(LOAD_X - half_width, 19), (LOAD_X + half_width, 19)],
        style.system_palette.pick(SystemColor::Foreground),
    ))?;

    const MAX_LOAD: f64 = 1.0;
    let colormap = Colormap::new_with_bounds(Some(ColormapType::Status).as_ref(), 0.0, MAX_LOAD);

    for (i, host) in (0..).zip(hosts) {
        let load: Option<f64> = loads.get(host).copied();

        debug!(
            "Processing host {} ({}, relative load: {})",
            i + 1,
            host,
            load.map(|l| l.to_string())
                .unwrap_or_else(|| "None".to_owned())
        );

        let vertical_step = infrastructure_summary.vertical_step.unwrap_or(20);
        let centered_y = 35 + vertical_step * i;

        debug!("Drawing hostname");
        let short_hostname = match infrastructure_summary.suffix {
            Some(ref suffix) => host.strip_suffix(suffix).unwrap_or(host),
            None => host,
        };
        new_root.draw(&Text::new(short_hostname, (15, centered_y), &host_font))?;

        debug!("Drawing status");
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
        new_root.draw(&Circle::new((STATUS_X, centered_y), 7, shape_style))?;
        new_root.draw(&Circle::new(
            (STATUS_X, centered_y),
            7,
            style.system_palette.pick(SystemColor::LightForeground),
        ))?;

        debug!("Drawing loadbar");
        let loadbar = Loadbar::new(
            (LOAD_X, centered_y),
            (40, 10),
            MAX_LOAD,
            load.unwrap_or(0.0),
            &style.system_palette,
            &colormap,
        );
        new_root.draw(&loadbar)?;
    }

    if let Some(ref format) = infrastructure_summary.last_update_format {
        let now: DateTime<Local> = now.with_timezone(&Local);
        new_root.draw(&Text::new(
            now.format(format).to_string(),
            (width as i32 - 10, height as i32 - 30),
            &footer_font,
        ))?;
    }

    Ok(())
}
