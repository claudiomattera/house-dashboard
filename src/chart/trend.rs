// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use tracing::*;

use std::cmp::Ord;
use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike, Utc, MAX_DATE, MIN_DATE};

use plotters::chart::{ChartBuilder, SeriesLabelPosition};
use plotters::drawing::{BitMapBackend, IntoDrawingArea};
use plotters::element::{Circle, PathElement, Text};
use plotters::series::LineSeries;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{Color, IntoFont};

use palette::Hsla;

use super::time_series_to_local_time;
use crate::configuration::StyleConfiguration;
use crate::configuration::TrendConfiguration;
use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::types::TimeSeries;

pub fn draw_trend_chart(
    time_seriess: HashMap<String, TimeSeries>,
    trend_configuration: TrendConfiguration,
    style: &StyleConfiguration,
    root: BitMapBackend,
) -> Result<(), DashboardError> {
    info!(
        "Drawing trend '{}'",
        trend_configuration.title.to_lowercase(),
    );

    let root = root.into_drawing_area();

    let title_font = (style.font.as_str(), 16.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));
    let label_font = (style.font.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));
    let legend_font = (style.font.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground));
    let value_font = (style.font.as_str(), 8.0 * style.font_scale)
        .into_font()
        .color(&style.system_palette.pick(SystemColor::Foreground))
        .pos(Pos::new(HPos::Right, VPos::Bottom));

    let mut indices = HashMap::<String, usize>::new();
    match trend_configuration.tag_values {
        Some(ref tag_values) => {
            for (index, name) in (0..).zip(tag_values.iter()) {
                indices.insert(name.to_owned(), index);
            }
        }
        None => {
            for (index, (name, _)) in (0..).zip(time_seriess.iter()) {
                indices.insert(name.to_owned(), index);
            }
        }
    };

    let mut min_x_utc = MAX_DATE.and_hms(0, 0, 0);
    let mut max_x_utc = MIN_DATE.and_hms(0, 0, 0);
    let mut min_y = std::f64::MAX;
    let mut max_y = std::f64::MIN;
    for (name, time_series) in time_seriess.iter() {
        if !indices.contains_key(name) {
            debug!(
                "Skipping unexpected time-series {} for range computation",
                name,
            );
            continue;
        }
        for (date, value) in time_series {
            min_x_utc = min_x_utc.min(*date);
            max_x_utc = max_x_utc.max(*date);
            min_y = min_y.min(value.clone().to_f64());
            max_y = max_y.max(value.clone().to_f64());
        }
    }

    // Increase maximal Y range to make space for the legend
    let top_padding = trend_configuration.top_padding.unwrap_or(0.0);
    max_y += top_padding * (max_y - min_y);

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

    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    debug!("Creating chart");

    let mut chart = ChartBuilder::on(&root)
        .caption(&trend_configuration.title, title_font)
        .margin(5)
        .x_label_area_size(20)
        .y_label_area_size(50)
        .build_ranged(min_x..max_x, min_y..max_y)?;

    debug!("Plotting time-series");

    type LocalTimeSeries = Vec<(DateTime<Local>, f64)>;

    let mut its: Vec<(String, LocalTimeSeries)> = time_seriess
        .iter()
        .map(|(s, ts)| (s.clone(), time_series_to_local_time(ts.clone()).into_iter().map(|(i, v)| (i, v.clone().to_f64())).collect()))
        .collect::<Vec<_>>();
    its.sort_by(|a, b| a.partial_cmp(b).expect("Invalid comparison"));

    for (name, time_series) in its.iter() {
        let index = match indices.get(name) {
            Some(index) => *index,
            None => {
                warn!("{}", DashboardError::UnexpectedTagValue(name.to_owned()));
                continue;
            }
        };

        chart
            .draw_series(LineSeries::new(
                time_series.iter().map(|(dt, value)| (*dt, *value)),
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
            chart.draw_series(time_series.iter().map(|(dt, value)| {
                Circle::new((*dt, *value), 3, style.series_palette.pick(index).filled())
            }))?;
        }

        if trend_configuration.draw_last_value.unwrap_or(false) {
            let last_reading = time_series.last().unwrap();
            let last_value = last_reading.1;
            let last_value_text = format!(
                "{0:.1$}",
                last_value,
                trend_configuration.precision.unwrap_or(0),
            );

            let last_value_coordinates = chart.backend_coord(last_reading);

            root.draw(&Text::new(
                last_value_text,
                last_value_coordinates,
                &value_font,
            ))?;
        }
    }

    debug!("Drawing axis");

    let ylabel = match (&trend_configuration.ylabel, &trend_configuration.yunit) {
        (Some(ylabel), Some(yunit)) => format!("{} [{}]", ylabel, yunit),
        (Some(ylabel), None) => ylabel.to_owned(),
        (None, _) => "".to_owned(),
    };

    let mut mesh = chart.configure_mesh();

    let mesh = if trend_configuration.draw_horizontal_grid.unwrap_or(false) {
        mesh.disable_x_mesh()
            .line_style_1(&style.system_palette.pick(SystemColor::Middle))
            .line_style_2(&Hsla::new(0.0, 0.0, 0.0, 0.0))
    } else {
        mesh.disable_mesh()
    };

    mesh.axis_style(&style.system_palette.pick(SystemColor::Foreground))
        .x_labels(trend_configuration.max_x_ticks.unwrap_or(4))
        .x_label_formatter(&|d| d.format(&trend_configuration.xlabel_format).to_string())
        .y_labels(trend_configuration.max_y_ticks.unwrap_or(5))
        .y_label_formatter(&|value| {
            format!("{0:.1$}", value, trend_configuration.precision.unwrap_or(0))
        })
        .y_desc(ylabel)
        .label_style(label_font)
        .draw()?;

    if !trend_configuration.hide_legend.unwrap_or(false) {
        debug!("Drawing legend");

        chart
            .configure_series_labels()
            .background_style(&style.system_palette.pick(SystemColor::LightBackground))
            .border_style(&style.system_palette.pick(SystemColor::LightForeground))
            .position(SeriesLabelPosition::UpperLeft)
            .label_font(legend_font)
            .draw()?;
    }

    Ok(())
}
