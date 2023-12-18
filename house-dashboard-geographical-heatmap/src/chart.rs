// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for generating chart

use tracing::{debug, info};

use std::collections::HashMap;
use std::hash::BuildHasher;

use itertools::Itertools;

use num_traits::{Bounded, FromPrimitive, Num, SaturatingAdd, SaturatingMul, Zero};

use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    coord::Shift,
    drawing::{DrawingArea, IntoDrawingArea},
    element::{PathElement, Polygon, Text},
    style::{
        colors::BLACK,
        text_anchor::{HPos, Pos, VPos},
        Color, IntoFont, RGBColor,
    },
};

use house_dashboard_common::{
    colormap::Colormap, configuration::StyleConfiguration, element::Colorbar, palette::SystemColor,
};

use crate::Error;
use crate::GeographicalHeatMapConfiguration;
use crate::GeographicalRegionConfiguration;

/// Draw an geographical heatmap chart
///
/// # Errors
///
/// Return and error when chart generation failed
pub fn draw_geographical_heatmap<S>(
    geographical_heatmap: &GeographicalHeatMapConfiguration,
    values: &HashMap<String, Option<f64>, S>,
    style: &StyleConfiguration,
    backend: BitMapBackend,
) -> Result<(), Error>
where
    S: BuildHasher,
{
    info!(
        "Drawing geographical heatmap '{}'",
        geographical_heatmap.title.to_lowercase()
    );

    let root = backend.into_drawing_area();
    root.fill(&style.system_palette.pick(SystemColor::Background))?;

    // Draw the title manually and create a new margin area
    let title_height = draw_title(geographical_heatmap.title.as_str(), style, &root)?;
    let new_root = root.margin(title_height, 0, 0, 60);
    let (width, height) = new_root.dim_in_pixel();

    let projected_regions = if geographical_heatmap.isometric.unwrap_or(false) {
        project_regions_to_isometric(&geographical_heatmap.regions)
    } else {
        project_regions(&geographical_heatmap.regions)
    };

    let normalized_projected_regions = normalize_regions(
        &projected_regions,
        (f64::from(width), f64::from(height)),
        (5_f64, 5_f64),
        (5_f64, 5_f64),
    );

    let colormap = Colormap::new_with_bounds_and_direction(
        geographical_heatmap.colormap.as_ref(),
        geographical_heatmap.bounds.0,
        geographical_heatmap.bounds.1,
        geographical_heatmap.reversed,
    )?;

    debug!("Drawing regions");
    for (name, path) in normalized_projected_regions
        .iter()
        .sorted_by_key(|pair| pair.0)
    {
        draw_region(
            geographical_heatmap,
            style,
            &colormap,
            values,
            name,
            path.as_slice(),
            &new_root,
        )?;
    }

    draw_colorbar(geographical_heatmap, style, colormap, &root)?;

    Ok(())
}

/// Draw title
fn draw_title<DB: DrawingBackend>(
    title: &str,
    style: &StyleConfiguration,
    root: &DrawingArea<DB, Shift>,
) -> Result<i32, Error> {
    let title_font = (style.font_name.as_str(), 16.0 * style.font_scale).into_font();
    let pos = Pos::new(HPos::Center, VPos::Top);

    let (width, _height) = root.dim_in_pixel();

    let (_box_width, box_height) = title_font.box_size(title).map_err(|_| Error::Font)?;
    let box_height = i32::try_from(box_height)?;
    let box_x = i32::try_from(width)? / 2;
    let box_y = box_height / 2;

    let vertical_skip = 5;

    root.draw(&Text::new(
        title,
        (box_x, box_y + vertical_skip),
        title_font
            .color(&style.system_palette.pick(SystemColor::Foreground))
            .pos(pos),
    ))?;

    Ok(box_height * 2)
}

/// Project regions to isometric view
fn project_regions_to_isometric(
    regions: &[GeographicalRegionConfiguration],
) -> HashMap<String, Vec<(f64, f64)>> {
    debug!("Computing projected regions");
    regions
        .iter()
        .map(|region| {
            let true_isometric_path: Vec<(f64, f64)> = region
                .coordinates
                .iter()
                .map(|&(x, y)| {
                    let (new_x, new_y, _new_z) = project_with_two_to_one_isometry(x, y, 0.0);
                    (new_x, new_y)
                })
                .collect();
            (region.name.clone(), true_isometric_path)
        })
        .collect()
}

/// Project regions
fn project_regions(
    regions: &[GeographicalRegionConfiguration],
) -> HashMap<String, Vec<(f64, f64)>> {
    debug!("Computing projected regions");
    regions
        .iter()
        .map(|region| {
            let true_isometric_path: Vec<(f64, f64)> = region.coordinates.clone();
            (region.name.clone(), true_isometric_path)
        })
        .collect()
}

/// Project a point with 2-to-1 isometry
#[allow(clippy::similar_names)]
pub fn project_with_two_to_one_isometry(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let size = 0.5;

    let origin_x = 0.0;
    let origin_y = 0.0;
    let origin_z = 0.0;

    let big_x_x = size * (1.0);
    let big_x_y = size * (1.0 / 2.0);
    let big_x_z = size * (-1.0 / (2.0 * 2.0_f64.sqrt()));

    let big_y_x = size * (-1.0);
    let big_y_y = size * (1.0 / 2.0);
    let big_y_z = size * (-1.0 / (2.0 * 2.0_f64.sqrt()));

    let big_z_x = size * (0.0);
    let big_z_y = size * (1.0);
    let big_z_z = size * (-1.0 / (2.0 * 2.0_f64.sqrt()));

    project(
        (x, y, z),
        (big_x_x, big_x_y, big_x_z),
        (big_y_x, big_y_y, big_y_z),
        (big_z_x, big_z_y, big_z_z),
        (origin_x, origin_y, origin_z),
    )
}

/// Project a point
fn project(
    point: (f64, f64, f64),
    big_x: (f64, f64, f64),
    big_y: (f64, f64, f64),
    big_z: (f64, f64, f64),
    origin: (f64, f64, f64),
) -> (f64, f64, f64) {
    #[allow(clippy::suspicious_operation_groupings)]
    let x = big_x.0 * point.0 + big_y.0 * point.1 + big_z.0 * point.2 + origin.0;

    #[allow(clippy::suspicious_operation_groupings)]
    let y = big_x.1 * point.0 + big_y.1 * point.1 + big_z.1 * point.2 + origin.1;

    #[allow(clippy::suspicious_operation_groupings)]
    let z = big_x.2 * point.0 + big_y.2 * point.1 + big_z.2 * point.2 + origin.2;

    (x, y, z)
}

/// Normalize regions
fn normalize_regions(
    regions: &HashMap<String, Vec<(f64, f64)>>,
    (width, height): (f64, f64),
    (top_margin, bottom_margin): (f64, f64),
    (left_margin, right_margin): (f64, f64),
) -> HashMap<String, Vec<(f64, f64)>> {
    let (min_x, max_x, min_y, max_y) = compute_all_regions_bounds(regions);

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
                name.clone(),
                path.iter()
                    .map(|&(x, y)| {
                        (
                            left_margin + slack_x / 2.0 + ((x - min_x) * ratio),
                            top_margin + slack_y / 2.0 + ((y - min_y) * ratio),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Compute the bounds for all regions
fn compute_all_regions_bounds(regions: &HashMap<String, Vec<(f64, f64)>>) -> (f64, f64, f64, f64) {
    let mut max_x = std::f64::MIN;
    let mut min_x = std::f64::MAX;
    let mut max_y = std::f64::MIN;
    let mut min_y = std::f64::MAX;
    for path in regions.values() {
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

/// Compute the bounds of a list of coordinates
pub fn bounds_of<T: Copy + Bounded + PartialOrd>(elements: &[T]) -> (T, T) {
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

/// Compute the centroid of a list of coordinates
pub fn centroid_of<T: Copy + Zero + Num + SaturatingMul + SaturatingAdd + FromPrimitive>(
    elements: &[(T, T)],
) -> (T, T) {
    let mut cx = T::zero();
    let mut cy = T::zero();
    let closed_elements: Vec<(T, T)> = elements
        .iter()
        .chain(elements.first().iter().copied())
        .copied()
        .collect();
    let paired = closed_elements.iter().skip(1).zip(elements.iter());
    for (&(x1, y1), &(x2, y2)) in paired {
        cx = cx.saturating_add(&(x1 + x2).saturating_mul(&(x1 * y2 - x2 * y1)));
        cy = cy.saturating_add(&(y1 + y2).saturating_mul(&(x1 * y2 - x2 * y1)));
    }
    let area = area_of(elements);

    #[allow(clippy::unwrap_used)]
    let cx = cx / (area * T::from_i64(6).unwrap());
    #[allow(clippy::unwrap_used)]
    let cy = cy / (area * T::from_i64(6).unwrap());
    (cx, cy)
}

/// Compute the area of a list of coordinates
fn area_of<T: Copy + Zero + Num + FromPrimitive>(elements: &[(T, T)]) -> T {
    let mut area = T::zero();
    let closed_elements: Vec<(T, T)> = elements
        .iter()
        .chain(elements.first().iter().copied())
        .copied()
        .collect();
    let paired = closed_elements.iter().skip(1).zip(elements.iter());
    for (&(x1, y1), &(x2, y2)) in paired {
        area = area + (x1 * y2 - x2 * y1);
    }

    #[allow(clippy::unwrap_used)]
    let denominator = T::from_i64(2).unwrap();

    area / denominator
}

/// Draw a region
fn draw_region<DB, S>(
    geographical_heatmap: &GeographicalHeatMapConfiguration,
    style: &StyleConfiguration,
    colormap: &Colormap,
    values: &HashMap<String, Option<f64>, S>,
    name: &str,
    path: &[(f64, f64)],
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error>
where
    DB: DrawingBackend,
    S: BuildHasher,
{
    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale).into_font();

    let value: Option<f64> = values.get(name).copied().flatten();
    debug!("Drawing region {}, value: {:?}", name, value);

    #[allow(clippy::cast_possible_truncation)]
    let path: Vec<(i32, i32)> = path.iter().map(|&(x, y)| (x as i32, y as i32)).collect();

    let closed_path: Vec<(i32, i32)> = path
        .last()
        .copied()
        .iter()
        .chain(path.iter())
        .copied()
        .collect();

    if let Some(value) = value {
        let color: RGBColor = colormap.get_color(value);
        root.draw(&Polygon::new(closed_path.clone(), color))?;

        let (cx, cy) = centroid_of(&path);
        let pos = Pos::new(HPos::Center, VPos::Center);
        root.draw(&Text::new(
            format!(
                "{0:.1$}",
                value,
                geographical_heatmap.precision.unwrap_or(0),
            ),
            (cx, cy),
            &label_font.color(&BLACK).pos(pos),
        ))?;
    }

    let indices = geographical_heatmap
        .colored_tag_values
        .as_ref()
        .map(|tag_values| {
            tag_values
                .iter()
                .enumerate()
                .map(|(index, name)| (name.clone(), index))
                .collect::<HashMap<String, usize>>()
        });

    let border_color = if let Some(ref indices) = indices {
        if let Some(index) = indices.get(name) {
            style.series_palette.pick(*index).stroke_width(2)
        } else {
            style
                .system_palette
                .pick(SystemColor::LightForeground)
                .stroke_width(1)
        }
    } else {
        style
            .system_palette
            .pick(SystemColor::LightForeground)
            .stroke_width(1)
    };

    root.draw(&PathElement::new(closed_path, border_color))?;

    Ok(())
}

/// Draw a colorbar
fn draw_colorbar<DB>(
    geographical_heatmap: &GeographicalHeatMapConfiguration,
    style: &StyleConfiguration,
    colormap: Colormap,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), Error>
where
    DB: DrawingBackend,
{
    debug!("Drawing colorbar");
    let label_font = (style.font_name.as_str(), 8.0 * style.font_scale).into_font();

    let (width, height) = root.dim_in_pixel();

    let right_margin = geographical_heatmap.right_margin.unwrap_or(55);

    #[allow(clippy::cast_possible_truncation)]
    let colorbar_width = (10.0 * style.font_scale) as i32;
    #[allow(clippy::cast_possible_truncation)]
    let colorbar_x = i32::try_from(width)?
        - colorbar_width
        - (f64::from(right_margin) * style.font_scale) as i32;

    let colorbar = Colorbar::new(
        (colorbar_x, 40),
        (colorbar_width, i32::try_from(height)? - 60),
        geographical_heatmap.bounds,
        geographical_heatmap.precision.unwrap_or(0),
        geographical_heatmap.unit.clone(),
        label_font,
        style.system_palette,
        colormap,
    );

    root.draw(&colorbar)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_of_list() {
        let path = vec![1, 4, 2, 5, 7, 2, 6, -2, 6, -1];
        let actual = bounds_of(&path);
        let expected = (-2, 7);
        assert_eq!(actual, expected);
    }

    #[test]
    fn area_of_square() {
        let path = vec![(0, 0), (0, 1), (1, 1), (1, 0)];
        let actual = area_of(&path);
        let expected = 1;
        assert_eq!(actual, expected);
    }

    #[test]
    fn area_of_triangle() {
        let path = vec![(0, 0), (1, 1), (2, 0)];
        let actual = area_of(&path);
        let expected = 1;
        assert_eq!(actual, expected);
    }

    #[test]
    fn area_of_path() {
        let path = vec![(0, 0), (0, 2), (1, 2), (1, 1), (2, 1), (2, 0)];
        let actual = area_of(&path);
        let expected = 3;
        assert_eq!(actual, expected);
    }

    #[test]
    fn centroid_of_square() {
        let path = vec![(0, 0), (0, 2), (2, 2), (2, 0)];
        let actual = centroid_of(&path);
        let expected = (1, 1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn centroid_of_triangle() {
        let path = vec![(0, 0), (1, 1), (2, 0)];
        let actual = centroid_of(&path);
        let expected = (1, 1 / 3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn centroid_of_path() {
        #[rustfmt::skip]
        let path = vec![(0, 0), (0, 2), (1, 2), (1, 1), (2, 1), (2, 0)];
        let actual = centroid_of(&path);
        let expected = (5 / 6, 5 / 6);
        assert_eq!(actual, expected);
    }
}
