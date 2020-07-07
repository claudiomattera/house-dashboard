// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use num_traits::{Bounded, FromPrimitive, Num, Zero};

mod geographicalheatmap;
mod temporalheatmap;
mod trend;

mod element;

pub use geographicalheatmap::draw_geographical_heat_map_chart;
pub use temporalheatmap::draw_temporal_heat_map_chart;
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

#[allow(dead_code)]
pub fn project_with_true_isometry(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let size = 1.0 / 2.0_f64.sqrt();

    let origin_x = 0.0;
    let origin_y = 0.0;
    let origin_z = 0.0;

    let big_x_x = (size / 2.0) * (3.0_f64.sqrt());
    let big_x_y = (size / 2.0) * 1.0;
    let big_x_z = (size / 2.0) * (-1.0 / 2.0_f64.sqrt());

    let big_y_x = (size / 2.0) * -(3.0_f64.sqrt());
    let big_y_y = (size / 2.0) * 1.0;
    let big_y_z = (size / 2.0) * (-1.0 / 2.0_f64.sqrt());

    let big_z_x = 0.0;
    let big_z_y = 2.0;
    let big_z_z = (size / 2.0) * (-1.0 / 2.0_f64.sqrt());

    project(
        (x, y, z),
        (big_x_x, big_x_y, big_x_z),
        (big_y_x, big_y_y, big_y_z),
        (big_z_x, big_z_y, big_z_z),
        (origin_x, origin_y, origin_z),
    )
}

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

fn project(
            point: (f64, f64, f64),
            big_x: (f64, f64, f64),
            big_y: (f64, f64, f64),
            big_z: (f64, f64, f64),
            origin: (f64, f64, f64),
        ) -> (f64, f64, f64) {
    let x = big_x.0 * point.0 + big_y.0 * point.1 + big_z.0 * point.2 + origin.0;
    let y = big_x.1 * point.0 + big_y.1 * point.1 + big_z.1 * point.2 + origin.1;
    let z = big_x.2 * point.0 + big_y.2 * point.1 + big_z.2 * point.2 + origin.2;

    (x, y, z)
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
