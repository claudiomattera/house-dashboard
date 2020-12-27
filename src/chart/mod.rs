// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use num_traits::{Bounded, FromPrimitive, Num, Zero};

use plotters::prelude::*;

mod geographicalmap;
mod trend;

pub use geographicalmap::draw_geographical_map_chart;
pub use trend::draw_trend_chart;

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

pub fn centroid_of<T: Copy + Zero + Num + FromPrimitive>(elements: &[(T, T)]) -> (T, T) {
    let mut cx = T::zero();
    let mut cy = T::zero();
    let closed_elements: Vec<(T, T)> = elements
        .iter()
        .chain(elements.first().iter().copied())
        .copied()
        .collect();
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
    let closed_elements: Vec<(T, T)> = elements.iter().chain(elements.first().iter().copied()).copied().collect();
    let paired = closed_elements.iter().skip(1).zip(elements.iter());
    for ((x1, y1), (x2, y2)) in paired {
        area = area + (*x1 * *y2 - *x2 * *y1);
    }
    area / T::from_i64(2).unwrap()
}

pub struct PaletteColorbrewerSet1;

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

pub struct PaletteDarkTheme;

impl Palette for PaletteDarkTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (0, 0, 0),
        (255, 255, 255),
        (32, 32, 32),
        (192, 192, 192),
    ];
}

#[allow(dead_code)]
pub struct PaletteLightTheme;

impl Palette for PaletteLightTheme {
    const COLORS: &'static [(u8, u8, u8)] = &[
        (255, 255, 255),
        (0, 0, 0),
        (192, 192, 192),
        (32, 32, 32),
    ];
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
        let path = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let actual = centroid_of(&path);
        let expected = (1.0, 1.0 / 3.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn centroid_of_path() {
        let path = vec![(0.0, 0.0), (0.0, 2.0), (1.0, 2.0), (1.0, 1.0), (2.0, 1.0), (2.0, 0.0)];
        let actual = centroid_of(&path);
        let expected = (5.0 / 6.0, 5.0 / 6.0);
        assert_eq!(actual, expected);
    }
}
