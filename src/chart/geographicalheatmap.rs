// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::collections::HashMap;

use plotters::drawing::{BitMapBackend, IntoDrawingArea};
use plotters::element::{PathElement, Polygon, Text};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{IntoFont, RGBColor, BLACK};

use crate::colormap::{Colormap, ColormapType};
use crate::error::DashboardError;
use crate::palette::SystemColor;
use crate::configuration::StyleConfiguration;

use super::element::colorbar::Colorbar;
use super::{bounds_of, centroid_of, project_with_two_to_one_isometry};

pub fn draw_geographical_heat_map_chart(
            values: HashMap<String, Option<f64>>,
            bounds: (f64, f64),
            precision: usize,
            colormap_type: Option<ColormapType>,
            reversed: Option<bool>,
            caption: &str,
            unit: &str,
            regions: HashMap<String, Vec<(f64, f64)>>,
            style: &StyleConfiguration,
            root: BitMapBackend,
        ) -> Result<(), DashboardError> {
    info!("Drawing geographical heat map \"{}\"", caption.to_lowercase());

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
            caption,
            (width as i32 / 2, 10),
            title_font.color(&style.system_palette.pick(SystemColor::Foreground)).pos(pos)
        )
    )?;

    let new_root = root.margin(0, 0, 0, 60);
    let (new_width, new_height) = new_root.dim_in_pixel();

    debug!("Computing projected regions");
    let projected_regions: HashMap::<String, Vec<(f64, f64)>> = regions
        .iter()
        .map(|(region, path)| {
            let true_isometric_path: Vec<(f64, f64)> = path
                .iter()
                .map(|(x, y)| {
                    let (new_x, new_y, _new_z) = project_with_two_to_one_isometry(*x as f64, *y as f64, 0.0);
                    (new_x, new_y)
                })
                .collect();
            (region.to_owned(), true_isometric_path)
        })
        .collect();

    let normalized_projected_regions = normalize_regions(
        projected_regions,
        (new_width as f64, new_height as f64),
        (5_f64, 5_f64),
        (5_f64, 5_f64),
    );

    debug!("Drawing regions");
    let colormap = Colormap::new_with_bounds_and_direction(colormap_type, bounds.0, bounds.1, reversed);
    for (name, path) in normalized_projected_regions {
        let value: Option<f64> = values.get(&name).copied().flatten();
        debug!("Drawing region {}, value: {:?}", name, value);
        let path: Vec<(i32, i32)> = path.iter().map(|(x, y)| (*x as i32, *y as i32)).collect();

        let closed_path: Vec<(i32, i32)> = path
            .last()
            .copied()
            .iter()
            .chain(path.iter())
            .copied()
            .collect();

        if let Some(value) = value {
            let color: RGBColor = colormap.get_color(value);
            new_root.draw(&Polygon::new(closed_path.clone(), &color))?;

            let (cx, cy) = centroid_of(&path);
            let pos = Pos::new(HPos::Center, VPos::Center);
            new_root.draw(
                &Text::new(
                    format!("{0:.1$}", value, precision),
                    (cx, cy),
                    &label_font.color(&BLACK).pos(pos),
                ),
            )?;
        }

        new_root.draw(&PathElement::new(closed_path, &style.system_palette.pick(SystemColor::LightForeground)))?;
    }

    debug!("Drawing colorbar");
    let colorbar = Colorbar::new(
        (width as i32 - 55, 40),
        (10, height as i32 - 60),
        bounds,
        precision,
        unit.to_owned(),
        label_font,
        style.system_palette,
        colormap,
    );

    root.draw(&colorbar)?;

    Ok(())
}

fn normalize_regions(
            regions: HashMap<String, Vec<(f64, f64)>>,
            (width, height): (f64, f64),
            (top_margin, bottom_margin): (f64, f64),
            (left_margin, right_margin): (f64, f64),
        ) -> HashMap<String, Vec<(f64, f64)>> {

    let (min_x, max_x, min_y, max_y) = compute_all_regions_bounds(&regions);

    let effective_width = width - left_margin - right_margin;
    let effective_height = height - top_margin - bottom_margin;

    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let ratio_x = effective_width / dx;
    let ratio_y = effective_height / dy;
    let ratio = ratio_x.min(ratio_y);

    let slack_x = effective_width - (dx * ratio);
    let slack_y = effective_height - (dy * ratio);

    regions
        .iter()
        .map(|(name, path)| {
            (
                name.to_owned(),
                path.iter().map(
                    |(x, y)| {
                        (
                            left_margin + slack_x / 2.0 + ((x - min_x) * ratio),
                            top_margin + slack_y / 2.0 + ((y - min_y) * ratio),
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
