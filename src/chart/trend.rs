// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::cmp::Ord;
use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike, Utc, MAX_DATE, MIN_DATE};

use plotters::chart::{ChartBuilder, SeriesLabelPosition};
use plotters::drawing::{BitMapBackend, IntoDrawingArea};
use plotters::element::{PathElement, Circle, Text};
use plotters::series::LineSeries;
use plotters::style::{Color, IntoFont};
use plotters::style::text_anchor::{HPos, Pos, VPos};

use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::types::TimeSeries;
use crate::configuration::StyleConfiguration;
use super::time_series_to_local_time;

pub fn draw_trend_chart(
            time_seriess: HashMap<String, TimeSeries>,
            caption: &str,
            ylabel: &Option<String>,
            ylabel_size: u32,
            xlabel_format: &str,
            precision: usize,
            draw_last_value: bool,
            hide_legend: bool,
            tag_values: Option<Vec<String>>,
            style: &StyleConfiguration,
            root: BitMapBackend,
        ) -> Result<(), DashboardError> {
    info!("Drawing trend");

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

    let mut min_x_utc = MAX_DATE.and_hms(0, 0, 0);
    let mut max_x_utc = MIN_DATE.and_hms(0, 0, 0);
    let mut min_y = std::f64::MAX;
    let mut max_y = std::f64::MIN;
    for time_series in time_seriess.values() {
        for (date, value) in time_series {
            min_x_utc = min_x_utc.min(*date);
            max_x_utc = max_x_utc.max(*date);
            min_y = min_y.min(*value);
            max_y = max_y.max(*value);
        }
    }

    // Add 20% more to the top to make space for the legend
    max_y += 2.0 * (max_y - min_y) / 10.0;

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
        .caption(caption, title_font)
        .margin(5)
        .x_label_area_size(20)
        .y_label_area_size(ylabel_size)
        .build_ranged(
            min_x..max_x,
            min_y..max_y,
        )?;

    debug!("Plotting time-series");

    type LocalTimeSeries = Vec<(DateTime<Local>, f64)>;

    let mut its: Vec<(String, LocalTimeSeries)> = time_seriess
        .iter()
        .map(|(s, ts)| (
            s.clone(),
            time_series_to_local_time(ts.clone()))
        )
        .collect::<Vec<_>>();
    its.sort_by(|a, b| a.partial_cmp(b).expect("Invalid comparison"));


    let mut indices = HashMap::<String, usize>::new();
    match tag_values {
        Some(tag_values) => {
            for (index, name) in (0..).zip(tag_values.iter()) {
                indices.insert(name.to_owned(), index);
            }
        }
        None => {
            for (index, (name, _)) in (0..).zip(its.iter()) {
                indices.insert(name.to_owned(), index);
            }
        }
    };

    for (name, time_series) in its.iter() {
        let index = match indices.get(name) {
            Some(index) => *index,
            None => {
                warn!("{}", DashboardError::UnexpectedTagValue(name.to_owned()));
                continue;
            }
        };

        chart
            .draw_series(
                LineSeries::new(
                    time_series.iter().map(|(dt, value)| (*dt, *value)),
                    style.series_palette.pick(index).stroke_width(3),
                )
            )?
            .label(name)
            .legend(move |(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    style.series_palette.pick(index).stroke_width(2),
                )
            });

        if style.draw_markers.unwrap_or(false) {
            chart.draw_series(
                time_series
                    .iter()
                    .map(
                        |(dt, value)|
                        Circle::new(
                            (*dt, *value),
                            3,
                            style.series_palette.pick(index).filled()
                        )
                    ),
            )?;
        }

        if draw_last_value {
            let last_reading = time_series.last().unwrap();
            let last_value = last_reading.1;
            let last_value_text = format!("{0:.1$}", last_value, precision);

            let last_value_coordinates = chart.backend_coord(&last_reading);

            root.draw(
                &Text::new(
                    last_value_text,
                    last_value_coordinates,
                    &value_font
                )
            )?;
        }
    }

    if !hide_legend {
        debug!("Drawing legend");

        chart
            .configure_series_labels()
            .background_style(&style.system_palette.pick(SystemColor::LightBackground))
            .border_style(&style.system_palette.pick(SystemColor::LightForeground))
            .position(SeriesLabelPosition::UpperLeft)
            .label_font(legend_font)
            .draw()?;
    }

    debug!("Drawing axis");

    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(&style.system_palette.pick(SystemColor::Foreground))
        .x_labels(4)
        .x_label_formatter(&|d| d.format(xlabel_format).to_string())
        .y_labels(5)
        .y_label_formatter(&|value| format!("{0:.1$}", value, precision))
        .y_desc(ylabel.as_ref().unwrap_or(&"".to_string()))
        .label_style(label_font)
        .draw()?;

    Ok(())
}
