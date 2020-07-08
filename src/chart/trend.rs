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
use plotters::element::PathElement;
use plotters::series::LineSeries;
use plotters::style::{Color, IntoFont};

use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::types::TimeSeries;
use crate::configuration::StyleConfiguration;

pub fn draw_trend_chart(
            time_seriess: HashMap<String, TimeSeries>,
            caption: &str,
            ylabel: &Option<String>,
            ylabel_size: u32,
            xlabel_format: &str,
            tag_values: Option<Vec<String>>,
            style: &StyleConfiguration,
            root: BitMapBackend,
        ) -> Result<(), DashboardError> {

    let root = root.into_drawing_area();

    let title_font = (style.font.as_str(), 16.0 * style.font_scale).into_font();
    let label_font = (style.font.as_str(), 8.0 * style.font_scale).into_font();
    let legend_font = (style.font.as_str(), 8.0 * style.font_scale).into_font();

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
        .caption(caption, title_font.color(&style.system_palette.pick(SystemColor::Foreground)))
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

        // chart.draw_series(
        //     time_series.iter()
        //         .map(|(dt, value)| Circle::new((*dt, *value), 2, style.series_palette.pick(index).filled())),
        // )?;
    }

    debug!("Drawing legend");

    chart
        .configure_series_labels()
        .background_style(&style.system_palette.pick(SystemColor::LightBackground))
        .border_style(&style.system_palette.pick(SystemColor::LightForeground))
        .position(SeriesLabelPosition::UpperLeft)
        .label_font(legend_font.color(&style.system_palette.pick(SystemColor::Foreground)))
        .draw()?;

    debug!("Drawing axis");

    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(&style.system_palette.pick(SystemColor::Foreground))
        .x_labels(4)
        .x_label_formatter(&|d| d.format(xlabel_format).to_string())
        .y_labels(5)
        .y_label_formatter(&|temperature| format!("{:.0}", temperature))
        .y_desc(ylabel.as_ref().unwrap_or(&"".to_string()))
        .label_style(label_font.color(&style.system_palette.pick(SystemColor::Foreground)))
        .draw()?;

    Ok(())
}

fn time_series_to_local_time(
            time_series: TimeSeries
        ) -> Vec<(DateTime<Local>, f64)> {
    time_series.iter().map(|(dt, v)| (dt.with_timezone(&Local), *v)).collect()
}
