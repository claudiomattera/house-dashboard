// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use tracing::*;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike, Utc, MAX_DATE, MIN_DATE};

use plotters::chart::ChartBuilder;
use plotters::drawing::{BitMapBackend, IntoDrawingArea};
use plotters::element::{Rectangle, Text};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{Color, IntoFont};

use crate::colormap::Colormap;
use crate::configuration::{StyleConfiguration, TemporalHeatMapConfiguration};
use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::types::TimeSeries;

use super::element::colorbar::Colorbar;
use super::time_series_to_local_time;

pub fn draw_temporal_heat_map_chart(
    time_series: TimeSeries,
    temporal_heatmap_configuration: TemporalHeatMapConfiguration,
    style: &StyleConfiguration,
    root: BitMapBackend,
) -> Result<(), DashboardError> {
    info!(
        "Drawing temporal heat map '{}'",
        temporal_heatmap_configuration.title.to_lowercase(),
    );

    let root = root.into_drawing_area();
    let (width, height) = root.dim_in_pixel();

    let title_font = (style.font.as_str(), 16.0 * style.font_scale).into_font();
    let label_font = (style.font.as_str(), 8.0 * style.font_scale).into_font();

    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    let mut min_x_utc = MAX_DATE.and_hms(0, 0, 0);
    let mut max_x_utc = MIN_DATE.and_hms(0, 0, 0);
    for (date, _value) in time_series.iter() {
        min_x_utc = min_x_utc.min(*date);
        max_x_utc = max_x_utc.max(*date);
    }

    let min_y = 0.0;
    let max_y = temporal_heatmap_configuration.period.max_y();

    let min_x = Utc
        .ymd(min_x_utc.year(), min_x_utc.month(), min_x_utc.day())
        .and_hms(min_x_utc.time().hour(), 0, 0)
        .checked_sub_signed(Duration::hours(1))
        .expect("Invalid duration")
        .with_timezone(&Local);
    let max_x = Utc
        .ymd(max_x_utc.year(), max_x_utc.month(), max_x_utc.day())
        .and_hms(max_x_utc.time().hour(), 0, 0)
        .checked_add_signed(Duration::hours(1))
        .expect("Invalid duration")
        .with_timezone(&Local);

    debug!("Plot X range: [{}, {}]", min_x, max_x);
    debug!("Plot Y range: [{}, {}]", min_y, max_y);

    // In order to make room for the colorbar, we need to set `margin_right()`
    // but that would make the title not centred.
    // So we must draw the title manually, and also add a top margin.
    let pos = Pos::new(HPos::Center, VPos::Top);
    root.draw(&Text::new(
        temporal_heatmap_configuration.title.clone(),
        (width as i32 / 2, 10),
        title_font
            .color(&style.system_palette.pick(SystemColor::Foreground))
            .pos(pos),
    ))?;

    let mut chart = ChartBuilder::on(&root)
        // .caption(caption, title_font.color(&style.system_palette.pick(SystemColor::Foreground)))
        .margin(5)
        .margin_top(40)
        .margin_right(70)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .build_ranged(min_x..max_x, min_y..max_y)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(&style.system_palette.pick(SystemColor::Foreground))
        .x_labels(3)
        .x_label_formatter(&|d| {
            d.format(temporal_heatmap_configuration.period.xlabel_format())
                .to_string()
        })
        .y_labels(4)
        .y_label_formatter(&|value| {
            format!(
                "{0:.1$}",
                value,
                temporal_heatmap_configuration.precision.unwrap_or(0),
            )
        })
        .x_desc(temporal_heatmap_configuration.period.xlabel())
        .y_desc(temporal_heatmap_configuration.period.ylabel())
        .label_style(label_font.color(&style.system_palette.pick(SystemColor::Foreground)))
        .draw()?;

    let time_series = time_series_to_local_time(time_series);

    let colormap = Colormap::new_with_bounds(
        temporal_heatmap_configuration.colormap.as_ref(),
        temporal_heatmap_configuration.bounds.0,
        temporal_heatmap_configuration.bounds.1,
    );

    let fragments: Vec<Rectangle<(DateTime<Local>, f64)>> = time_series
        .iter()
        .map(|(instant, value)| {
            let ((x1, x2), (y1, y2)) = temporal_heatmap_configuration
                .period
                .instant_to_rectangle(*instant);
            Rectangle::new(
                [(x1, y1 as f64), (x2, y2 as f64)],
                colormap.get_color(value.clone().to_f64()).filled(),
            )
        })
        .collect();

    chart.draw_series(fragments)?;

    let colorbar = Colorbar::new(
        (width as i32 - 55, 40),
        (10, height as i32 - 60),
        temporal_heatmap_configuration.bounds,
        temporal_heatmap_configuration.precision.unwrap_or(0),
        temporal_heatmap_configuration.unit,
        label_font,
        style.system_palette,
        colormap,
    );

    root.draw(&colorbar)?;

    Ok(())
}
