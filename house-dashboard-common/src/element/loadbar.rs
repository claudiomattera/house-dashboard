// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for generating load bars

use std::iter::{once, Once};

use plotters::element::{Drawable, PointCollection};

use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use crate::{
    colormap::Colormap,
    palette::{SystemColor, SystemPalette},
};

/// Loadbar
pub struct Loadbar<'a> {
    /// Position
    position: (i32, i32),

    /// Size
    size: (i32, i32),

    /// Max load
    max: f64,

    /// Actual load
    value: f64,

    /// System palette
    system_palette: &'a SystemPalette,

    /// Colormap
    colormap: &'a Colormap,

    /// Colormap steps
    n: i32,
}

impl<'a> Loadbar<'a> {
    /// Create a new load bar
    #[must_use]
    pub fn new(
        position: (i32, i32),
        size: (i32, i32),
        max: f64,
        value: f64,
        system_palette: &'a SystemPalette,
        colormap: &'a Colormap,
    ) -> Self {
        Self {
            position,
            size,
            max,
            value,
            system_palette,
            colormap,
            n: 61,
        }
    }
}

impl<'a> PointCollection<'a, (i32, i32)> for &'a Loadbar<'a> {
    type Point = &'a (i32, i32);
    type IntoIter = Once<&'a (i32, i32)>;

    fn point_iter(self) -> Self::IntoIter {
        once(&self.position)
    }
}

impl<'a, DB: DrawingBackend> Drawable<DB> for Loadbar<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = pos.next() {
            let upper_left = (x - self.size.0 / 2 - 1, y - self.size.1 / 2 - 1);
            let bottom_right = (x + self.size.0 / 2 + 1, y + self.size.1 / 2);
            backend.draw_rect(
                upper_left,
                bottom_right,
                &self.system_palette.pick(SystemColor::LightForeground),
                false,
            )?;

            let width = self.size.0;

            let step = f64::from(width) / f64::from(self.n);
            let last = ((f64::from(self.n) * self.value) / self.max).trunc() as i32;
            for i in 0..self.n {
                if i >= last {
                    break;
                }
                let upper_left = (
                    x - self.size.0 / 2 + (f64::from(i) * step).trunc() as i32,
                    y - self.size.1 / 2,
                );
                let bottom_right = (
                    x - self.size.0 / 2 + (f64::from(i + 1) * step).trunc() as i32,
                    y + self.size.1 / 2,
                );
                let value = f64::from(i) * self.max / f64::from(self.n);
                let color = self.colormap.get_color(value);
                backend.draw_rect(upper_left, bottom_right, &color, true)?;
            }
        }
        Ok(())
    }
}
