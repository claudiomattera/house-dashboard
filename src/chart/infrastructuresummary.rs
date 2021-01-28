// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use tracing::*;

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};

use plotters::drawing::{BitMapBackend, IntoDrawingArea};
use plotters::style::{Color, IntoFont, ShapeStyle};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::element::{Circle, Text};

use crate::colormap::{Colormap, ColormapType};
use crate::configuration::InfrastructureSummaryConfiguration;
use crate::configuration::StyleConfiguration;
use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::types::TimeSeries;
use super::element::loadbar::Loadbar;

pub fn draw_infrastructure_summary(
            infrastructure_summary: InfrastructureSummaryConfiguration,
            hosts: HashSet<String>,
            loads: HashMap<String, TimeSeries>,
            style: &StyleConfiguration,
            root: BitMapBackend,
        ) -> Result<(), DashboardError> {
    info!("Drawing infrastructure summary");

    let title_font = (style.font.as_str(), 16.0 * style.font_scale).into_font();
    let label_font = (style.font.as_str(), 8.0 * style.font_scale).into_font();

    let root = root.into_drawing_area();
    let (width, height) = root.dim_in_pixel();

    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // In order to make room for the colorbar, we need to set `margin_right()`
    // but that would make the title not centred.
    // So we must draw the title manually, and also create a new margin area.
    let pos = Pos::new(HPos::Center, VPos::Top);
    root.draw(
        &Text::new(
            "INFRASTRUCTURE",
            (width as i32 / 2, 10),
            title_font.color(&style.system_palette.pick(SystemColor::Foreground)).pos(pos)
        )
    )?;

    let new_root = root.margin(30, 0, 0, 0);
    let (_new_width, _new_height) = new_root.dim_in_pixel();

    let mut hosts = hosts.iter().collect::<Vec<&String>>();
    hosts.sort();

    let header_pos = Pos::new(HPos::Center, VPos::Top);
    let header_font = label_font.color(&style.system_palette.pick(SystemColor::Foreground)).pos(header_pos);

    let host_pos = Pos::new(HPos::Left, VPos::Center);
    let host_font = label_font.color(&style.system_palette.pick(SystemColor::Foreground)).pos(host_pos);

    let footer_pos = Pos::new(HPos::Right, VPos::Bottom);
    let footer_font = label_font.color(&style.system_palette.pick(SystemColor::Foreground)).pos(footer_pos);

    const STATUS_X: i32 = 220;
    const LOAD_X: i32 = 280;

    new_root.draw(&Text::new("HOST", (50, 10), &header_font))?;
    new_root.draw(&Text::new("STATUS", (STATUS_X, 10), &header_font))?;
    new_root.draw(&Text::new("LOAD", (LOAD_X, 10), &header_font))?;

    const MAX_LOAD: f64 = 1.0;
    let colormap = Colormap::new_with_bounds(Some(ColormapType::Status), 0.0, MAX_LOAD);

    for (i, host) in (0..).zip(hosts) {
        let load: Option<f64> = loads
            .get(host)
            .map(|loads| loads.last())
            .flatten()
            .map(|(_instant, value)| *value);

        debug!(
            "Processing host {} ({}, relative load: {})",
            i + 1,
            host,
            load.map(|l| l.to_string()).unwrap_or_else(|| "None".to_owned())
        );

        let centered_y = 35 + 22*i;

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
        new_root.draw(&Circle::new((STATUS_X, centered_y), 7, &style.system_palette.pick(SystemColor::LightForeground)))?;

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

    if let Some(format) = infrastructure_summary.last_update_format {
        let now: DateTime<Local> = Local::now();
        new_root.draw(
            &Text::new(
                now.format(&format).to_string(),
                (width as i32 - 10, height as i32 - 30),
                &footer_font
            )
        )?;
    }

    Ok(())
}
