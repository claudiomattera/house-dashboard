// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::collections::HashMap;
use std::cmp::Ord;

use chrono::{MIN_DATE, MAX_DATE, Datelike, DateTime, Duration, Local, Timelike, TimeZone, Utc};

use num_traits::{Bounded, FromPrimitive, Num, Zero};

use plotters::prelude::*;
use plotters::style::text_anchor::{Pos, HPos, VPos};

use crate::error::DashboardError;
use crate::colormap::Colormap;
use crate::types::TimeSeries;

pub fn draw_chart(
            time_seriess: HashMap<String, TimeSeries>,
            caption: &str,
            ylabel: &Option<String>,
            ylabel_size: u32,
            xlabel_format: &str,
            tag_values: Option<Vec<String>>,
            root: impl IntoDrawingArea<ErrorType = DashboardError>,
        ) -> Result<(), DashboardError> {

    let root = root.into_drawing_area();

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
            .checked_sub_signed(Duration::hours(1))
            .expect("Invalid duration")
            .with_timezone(&Local);
    let max_x = Utc.ymd(max_x_utc.year(), max_x_utc.month(), max_x_utc.day())
            .and_hms(max_x_utc.time().hour(), 0, 0)
            .checked_add_signed(Duration::hours(1))
            .expect("Invalid duration")
            .with_timezone(&Local);

    debug!("Plot X range: [{}, {}]", min_x, max_x);
    debug!("Plot Y range: [{}, {}]", min_y, max_y);

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
    its.sort_by(|a, b| a.partial_cmp(b).expect("Invalid comparison"));


    let mut indices = HashMap::<String, usize>::new();
    match tag_values {
        Some(tag_values) => {
            for (index, name) in (0..).zip(tag_values.iter()) {
                indices.insert(name.to_owned(), index);
            }
        },
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

pub fn draw_geographical_map_chart(
            values: HashMap<String, Option<f64>>,
            bounds: (f64, f64),
            caption: &str,
            unit: &str,
            regions: HashMap<String, Vec<(f64, f64)>>,
            mut root: impl IntoDrawingArea<ErrorType = DashboardError>,
        ) -> Result<(), DashboardError> {
    info!("Drawing geographical map");

    let title_font = ("Apple ][", 16).into_font();
    let label_font = ("Apple ][", 8).into_font();

    let mut max_x = std::f64::MIN;
    let mut min_x = std::f64::MAX;
    let mut max_y = std::f64::MIN;
    let mut min_y = std::f64::MAX;
    for (_, path) in regions.iter() {
        let xs: Vec<f64> = path.iter().map(|p| p.0).collect();
        let ys: Vec<f64> = path.iter().map(|p| p.1).collect();
        let (local_min_x, local_max_x) = bounds_of(&xs);
        let (local_min_y, local_max_y) = bounds_of(&ys);
        max_x = max_x.max(local_max_x);
        max_y = max_y.max(local_max_y);
        min_x = min_x.min(local_min_x);
        min_y = min_y.min(local_min_y);
    }

    let (width, height) = root.get_size();
    let (width, height) = (width as i32, height as i32);
    let top_margin = 40;
    let bottom_margin = 15;
    let left_margin = 50;
    let right_margin = 15;
    let chart_width = width - left_margin - right_margin;
    let chart_height = height - top_margin - bottom_margin;
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let ratio_x = chart_width as f64 / dx;
    let ratio_y = chart_height as f64 / dy;
    let ratio = ratio_x.min(ratio_y);

    let slack_x = chart_width - (dx * ratio) as i32;
    let slack_y = chart_height - (dy * ratio) as i32;

    let normalized_regions: HashMap<String, Vec<(i32, i32)>> = regions.iter()
        .map(|(name, path)| {
            (name.to_owned(), path.iter().map(
                |(x, y)| {
                    (slack_x / 2 + ((x - min_x) * ratio) as i32, top_margin + slack_y / 2 + ((y - min_y) * ratio) as i32)
                }).collect())
        })
        .collect();

    let colormap = Colormap::new_with_bounds("coolwarm", bounds.0, bounds.1)?;

    type BasicPalette = PaletteDarkTheme;
    let (title_width, _title_height) = root.estimate_text_size(caption, &title_font)?;
    root.draw_text(caption, &title_font.color(&BasicPalette::pick(1)), ((320 - title_width as i32) / 2, 10))?;

    for (name, path) in normalized_regions {
        let value: Option<f64> = values.get(&name).map(|v| *v).flatten();
        info!("Drawing region {}, value: {:?}", name, value);
        draw_region(value, &colormap, path, &label_font, &mut root)?;
    }

    root.draw_rect((width - left_margin - 10, top_margin), (width - left_margin, height - bottom_margin), &BasicPalette::pick(1), false)?;
    let n = 61;
    let label_count = 4;
    let label_step = n / label_count;
    let step = chart_height as f64 / (n as f64);
    for i in 0..n {
        let upper_left = (width - left_margin - 10, top_margin + (i as f64 * step) as i32);
        let bottom_right = (width - left_margin, top_margin + ((i + 1) as f64 * step) as i32);
        let value = bounds.0 + (n - i) as f64 * (bounds.1 - bounds.0) / n as f64;
        let color = colormap.get_color(value);
        root.draw_rect(upper_left, bottom_right, &color, true)?;

        if i % label_step == 0 {
            let pos = Pos::new(HPos::Left, VPos::Center);
            let position = (width - left_margin + 5, (upper_left.1 + bottom_right.1) / 2);
            root.draw_text(&format!("{:.0}{}", value, unit), &label_font.color(&BasicPalette::pick(1)).pos(pos), position)?;
        }
    }

    Ok(())
}

fn time_series_to_local_time(
            time_series: TimeSeries
        ) -> Vec<(DateTime<Local>, f64)> {
    time_series.iter().map(|(dt, v)| (dt.with_timezone(&Local), *v)).collect()
}

fn bounds_of<T: Copy + Bounded + PartialOrd>(elements: &[T]) -> (T, T) {
    let mut max = T::min_value();
    let mut min = T::max_value();
    for element in elements {
        if *element < min {
            min = *element;
        }
        if *element > max {
            max = *element;
        }
    }
    (min, max)
}

fn centroid_of<T: Copy + Zero + Num + FromPrimitive>(elements: &[(T, T)]) -> (T, T) {
    let mut cx = T::zero();
    let mut cy = T::zero();
    let closed_elements: Vec<(T, T)> = elements.iter().chain(elements.first().iter().map(|v| *v)).map(|v| *v).collect();
    let paired = closed_elements.iter().skip(1).zip(elements.iter());
    for ((x1, y1), (x2, y2)) in paired {
        cx = cx + (*x1 + *x2) * (*x1 * *y2 - *x2 * *y1);
        cy = cy + (*y1 + *y2) * (*x1 * *y2 - *x2 * *y1);
    }
    let area = area_of(elements);
    cx = cx / (area * T::from_i64(6).unwrap());
    cy = cy / (area * T::from_i64(6).unwrap());
    (cx, cy)
}

fn area_of<T: Copy + Zero + Num + FromPrimitive>(elements: &[(T, T)]) -> T {
    let mut area = T::zero();
    let closed_elements: Vec<(T, T)> = elements.iter().chain(elements.first().iter().map(|v| *v)).map(|v| *v).collect();
    let paired = closed_elements.iter().skip(1).zip(elements.iter());
    for ((x1, y1), (x2, y2)) in paired {
        area = area + (*x1 * *y2 - *x2 * *y1);
    }
    area / T::from_i64(2).unwrap()
}

fn draw_region(
            value: Option<f64>,
            colormap: &Colormap,
            path: Vec<(i32, i32)>,
            label_font: &FontDesc,
            root: &mut impl IntoDrawingArea<ErrorType = DashboardError>,
        ) -> Result<(), DashboardError> {
    if let Some(value) = value {
        let color = colormap.get_color(value);
        root.fill_polygon(path.clone(), &color)?;
        let (cx, cy) = centroid_of(&path);
        type BasicPalette = PaletteDarkTheme;
        root.draw_text(&format!("{:.0}", value), &label_font.color(&BasicPalette::pick(0)), (cx, cy))?;
    }
    let closed_path: Vec<(i32, i32)> = path.last().map(|v| *v).iter().chain(path.iter()).map(|v| *v).collect();
    root.draw_path(closed_path, &RGBColor(255, 255, 255))?;
    Ok(())
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
