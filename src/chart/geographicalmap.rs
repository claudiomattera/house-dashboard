// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::collections::HashMap;

use plotters::drawing::IntoDrawingArea;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{FontDesc, IntoFont, Palette, RGBColor};

use crate::colormap::Colormap;
use crate::error::DashboardError;

use super::{bounds_of, centroid_of, PaletteDarkTheme};

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

    let parameters = Parameters::new(
        root.get_size(),
        40,
        15,
        50,
        15,
        label_font,
    );

    debug!("Computing normalized regions");
    let normalized_regions = normalize_regions(regions, &parameters);

    debug!("Filling background");
    root.fill_polygon(
        vec![(0, 0), (parameters.width, 0), (parameters.width, parameters.height), (0, parameters.height)],
        &BasicPalette::pick(0),
    )?;

    debug!("Drawing title");
    let (title_width, _title_height) = root.estimate_text_size(caption, &title_font)?;
    root.draw_text(
        caption,
        &title_font.color(&BasicPalette::pick(1)),
        ((320 - title_width as i32) / 2, 10),
    )?;

    debug!("Drawing regions");
    let colormap = Colormap::new_with_bounds("coolwarm", bounds.0, bounds.1)?;
    for (name, path) in normalized_regions {
        let value: Option<f64> = values.get(&name).copied().flatten();
        debug!("Drawing region {}, value: {:?}", name, value);
        draw_region(value, &colormap, path, &parameters.label_font, &mut root)?;
    }

    debug!("Drawing colorbar");
    draw_colorbar(&parameters, bounds, unit, &colormap, &mut root)?;

    Ok(())
}

type BasicPalette = PaletteDarkTheme;

struct Parameters<'a> {
    pub width: i32,
    pub height: i32,
    pub chart_width: i32,
    pub chart_height: i32,
    pub top_margin: i32,
    pub bottom_margin: i32,
    pub left_margin: i32,
    pub right_margin: i32,
    pub label_font: FontDesc<'a>,
}

impl<'a> Parameters<'a> {
    pub fn new(
                resolution: (u32, u32),
                top_margin: i32,
                bottom_margin: i32,
                left_margin: i32,
                right_margin: i32,
                label_font: FontDesc<'a>,
            ) -> Self {
        let width = resolution.0 as i32;
        let height = resolution.1 as i32;
        Parameters {
            width,
            height,
            chart_width: width - left_margin - right_margin,
            chart_height: height - top_margin - bottom_margin,
            top_margin,
            bottom_margin,
            left_margin,
            right_margin,
            label_font,
        }
    }
}

fn normalize_regions(
            regions: HashMap<String, Vec<(f64, f64)>>,
            parameters: &Parameters,
        ) -> HashMap<String, Vec<(i32, i32)>> {

    let (min_x, max_x, min_y, max_y) = compute_all_regions_bounds(&regions);

    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let ratio_x = parameters.chart_width as f64 / dx;
    let ratio_y = parameters.chart_height as f64 / dy;
    let ratio = ratio_x.min(ratio_y);

    let slack_x = parameters.chart_width - (dx * ratio) as i32;
    let slack_y = parameters.chart_height - (dy * ratio) as i32;

    regions
        .iter()
        .map(|(name, path)| {
            (
                name.to_owned(),
                path.iter().map(
                    |(x, y)| {
                        (
                            slack_x / 2 + ((x - min_x) * ratio) as i32,
                            parameters.top_margin + slack_y / 2 + ((y - min_y) * ratio) as i32,
                        )
                    }).collect(),
            )
        })
        .collect()
}

fn compute_all_regions_bounds(regions: &HashMap<String, Vec<(f64, f64)>>) -> (f64, f64, f64, f64) {
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
    (min_x, max_x, min_y, max_y)
}

fn draw_region(
            value: Option<f64>,
            colormap: &Colormap,
            path: Vec<(i32, i32)>,
            label_font: &FontDesc,
            root: &mut impl IntoDrawingArea<ErrorType = DashboardError>,
        ) -> Result<(), DashboardError> {
    // Always draw region value in black, regardless of palette.
    // The background depends on the colormap, not on the palette.
    const BLACK: RGBColor = RGBColor(0, 0, 0);

    if let Some(value) = value {
        let color = colormap.get_color(value);
        root.fill_polygon(path.clone(), &color)?;
        let (cx, cy) = centroid_of(&path);
        root.draw_text(
            &format!("{:.0}", value),
            &label_font.color(&BLACK),
            (cx, cy),
        )?;
    }
    let closed_path: Vec<(i32, i32)> = path
        .last()
        .copied()
        .iter()
        .chain(path.iter())
        .copied()
        .collect();
    root.draw_path(closed_path, &BasicPalette::pick(3))?;
    Ok(())
}

fn draw_colorbar(
            parameters: &Parameters,
            bounds: (f64, f64),
            unit: &str,
            colormap: &Colormap,
            root: &mut impl IntoDrawingArea<ErrorType = DashboardError>,
        ) -> Result<(), DashboardError> {
    root.draw_rect(
        (
            parameters.width - parameters.left_margin - 10 - 1,
            parameters.top_margin - 1,
        ),
        (
            parameters.width - parameters.left_margin + 1,
            parameters.height - parameters.bottom_margin + 1,
        ),
        &BasicPalette::pick(3),
        false,
    )?;
    let n = 61;
    let label_count = 4;
    let label_step = n / label_count;
    let step = parameters.chart_height as f64 / (n as f64);
    for i in 0..n {
        let upper_left = (
            parameters.width - parameters.left_margin - 10,
            parameters.top_margin + (i as f64 * step) as i32,
        );
        let bottom_right = (
            parameters.width - parameters.left_margin,
            parameters.top_margin + ((i + 1) as f64 * step) as i32,
        );
        let value = bounds.0 + (n - i) as f64 * (bounds.1 - bounds.0) / n as f64;
        let color = colormap.get_color(value);
        root.draw_rect(upper_left, bottom_right, &color, true)?;

        if i % label_step == 0 {
            let pos = Pos::new(HPos::Left, VPos::Center);
            let position = (
                parameters.width - parameters.left_margin + 5,
                (upper_left.1 + bottom_right.1) / 2,
            );
            root.draw_text(
                &format!("{:.0}{}", value, unit),
                &parameters.label_font.color(&BasicPalette::pick(1)).pos(pos),
                position,
            )?;
        }
    }
    Ok(())
}
