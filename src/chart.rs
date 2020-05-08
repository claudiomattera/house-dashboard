// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::path::Path;

use std::collections::HashMap;

use chrono::{MIN_DATE, MAX_DATE, Datelike, DateTime, Duration, Local, Timelike, TimeZone, Utc};

use plotters::prelude::*;

use crate::types::TimeSeries;


#[derive(Debug)]
pub enum BackendType<'a> {
    FrameBuffer((&'a mut [u8], u32, u32)),
    File(&'a Path, (u32, u32)),
}


pub fn draw_chart(
            time_seriess: HashMap<String, TimeSeries>,
            caption: &str,
            ylabel: &Option<String>,
            ylabel_size: u32,
            xlabel_format: &str,
            backend_type: BackendType,
        ) -> Result<(), Box<dyn std::error::Error>> {
    let title_font = ("Apple ][", 16).into_font();
    let label_font = ("Apple ][", 8).into_font();
    let legend_font = ("Apple ][", 8).into_font();

    type BasicPalette = PaletteDarkTheme;
    type SeriesPalette = PaletteColorbrewerSet1;

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

    let min_x = Utc.ymd(min_x_utc.year(), min_x_utc.month(), min_x_utc.day())
            .and_hms(min_x_utc.time().hour(), 0, 0)
            .checked_sub_signed(Duration::hours(1)).unwrap()
            .with_timezone(&Local);
    let max_x = Utc.ymd(max_x_utc.year(), max_x_utc.month(), max_x_utc.day())
            .and_hms(max_x_utc.time().hour(), 0, 0)
            .checked_add_signed(Duration::hours(1)).unwrap()
            .with_timezone(&Local);

    debug!("Plot X range: [{}, {}]", min_x, max_x);
    debug!("Plot Y range: [{}, {}]", min_y, max_y);

    let root = match backend_type {
        BackendType::FrameBuffer((buffer, width, height)) => BitMapBackend::with_buffer(buffer, (width, height)),
        BackendType::File(path, resolution) => BitMapBackend::new(path, resolution),
    }.into_drawing_area();

    root.fill(&BasicPalette::pick(0))?;

    debug!("Creating chart");

    let mut chart = ChartBuilder::on(&root)
        .caption(caption, title_font.color(&BasicPalette::pick(1)))
        .margin(5)
        .x_label_area_size(20)
        .y_label_area_size(ylabel_size)
        .build_ranged(
            min_x..max_x,
            min_y..max_y,
        )?;

    debug!("Plotting time-series");

    type LocalTimeSeries = Vec<(DateTime<Local>, f64)>;

    let mut its: Vec<(String, LocalTimeSeries)> = time_seriess.iter()
        .map(|(s, ts)| (
            s.clone(),
            time_series_to_local_time(ts.clone()))
        )
        .collect::<Vec<_>>();
    its.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for (index, (name, time_series)) in (0..).zip(its.iter()) {

        chart.draw_series(
            LineSeries::new(
                time_series.iter().map(|(dt, value)| (*dt, *value)),
                SeriesPalette::pick(index).stroke_width(3),
            )
        )?
        .label(name)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], SeriesPalette::pick(index).stroke_width(2)))
        ;

        // chart.draw_series(
        //     time_series.iter()
        //         .map(|(dt, value)| Circle::new((*dt, *value), 2, SeriesPalette::pick(index).filled())),
        // )?;
    }

    debug!("Drawing legend");

    chart
        .configure_series_labels()
        .background_style(&BasicPalette::pick(2))
        .border_style(&BasicPalette::pick(3))
        .position(SeriesLabelPosition::UpperLeft)
        .label_font(legend_font.color(&BasicPalette::pick(1)))
        .draw()?;

    debug!("Drawing axis");

    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(&BasicPalette::pick(1))
        .x_labels(4)
        .x_label_formatter(&|d| d.format(xlabel_format).to_string())
        .y_labels(5)
        .y_label_formatter(&|temperature| format!("{:.0}", temperature))
        .y_desc(ylabel.as_ref().unwrap_or(&"".to_string()))
        .label_style(label_font.color(&BasicPalette::pick(1)))
        .draw()?;

    Ok(())
}

fn time_series_to_local_time(
            time_series: TimeSeries
        ) -> Vec<(DateTime<Local>, f64)> {
    time_series.iter().map(|(dt, v)| (dt.with_timezone(&Local), *v)).collect()
}

struct PaletteColorbrewerSet1;

impl Palette for PaletteColorbrewerSet1 {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (228, 26, 28),
        (55, 126, 184),
        (77, 175, 74),
        (152, 78, 163),
        (255, 127, 0),
        (255, 255, 51),
        (166, 86, 40),
        (247, 129, 191),
        (153, 153, 153),
    ];
}


struct PaletteDarkTheme;

impl Palette for PaletteDarkTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (0, 0, 0),
        (255, 255, 255),
        (32, 32, 32),
        (192, 192, 192),
    ];
}
