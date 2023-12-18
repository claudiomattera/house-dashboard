// Copyright Claudio Mattera 2022.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Data types for generating color bars

use std::iter::{once, Once};

use plotters::{
    backend::DrawingBackend,
    element::{Drawable, PointCollection},
    style::{
        text_anchor::{HPos, Pos, VPos},
        FontDesc,
    },
};

use plotters_backend::{BackendCoord, DrawingErrorKind};

use crate::{
    colormap::Colormap,
    palette::{SystemColor, SystemPalette},
};

/// A colorbar
pub struct Colorbar<'a> {
    /// Position
    position: (i32, i32),

    /// Size
    size: (i32, i32),

    /// Colormap bounds
    bounds: (f64, f64),

    /// Precision
    precision: usize,

    /// Unit
    unit: String,

    /// Font
    label_font: FontDesc<'a>,

    /// System palette
    system_palette: SystemPalette,

    /// Colormap
    colormap: Colormap,

    /// Number of steps to divide the colormap
    n: i32,
}

impl<'a> Colorbar<'a> {
    /// Create a new colorbar
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        position: (i32, i32),
        size: (i32, i32),
        bounds: (f64, f64),
        precision: usize,
        unit: String,
        label_font: FontDesc<'a>,
        system_palette: SystemPalette,
        colormap: Colormap,
    ) -> Self {
        Self {
            position,
            size,
            bounds,
            precision,
            unit,
            label_font,
            system_palette,
            colormap,
            n: 61,
        }
    }
}

impl<'a> PointCollection<'a, (i32, i32)> for &'a Colorbar<'a> {
    type Point = &'a (i32, i32);
    type IntoIter = Once<&'a (i32, i32)>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.position)
    }
}

impl<'a, DB: DrawingBackend> Drawable<DB> for Colorbar<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        _pos: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        backend.draw_rect(
            (self.position.0 - 1, self.position.1 - 1),
            (
                self.position.0 + self.size.0 + 1,
                self.position.1 + self.size.1 + 1,
            ),
            &self.system_palette.pick(SystemColor::LightForeground),
            false,
        )?;

        let height = self.size.1;

        let label_count = 4;
        let label_step = self.n / label_count;
        let step = f64::from(height) / (f64::from(self.n));
        for i in 0..self.n {
            let upper_left = (
                self.position.0,
                self.position.1 + (f64::from(i) * step) as i32,
            );
            let bottom_right = (
                self.position.0 + self.size.0 + 1,
                self.position.1 + (f64::from(i + 1) * step) as i32 + 1,
            );
            let value = self.bounds.0
                + f64::from(self.n - i - 1) * (self.bounds.1 - self.bounds.0)
                    / f64::from(self.n - 1);
            let color = self.colormap.get_color(value);
            backend.draw_rect(upper_left, bottom_right, &color, true)?;

            if i % label_step == 0 {
                let pos = Pos::new(HPos::Left, VPos::Center);
                let position = (
                    self.position.0 + self.size.0 + 5,
                    (upper_left.1 + bottom_right.1) / 2,
                );
                backend.draw_text(
                    &format!("{0:.1$}{2}", value, self.precision, self.unit),
                    &self
                        .label_font
                        .color(&self.system_palette.pick(SystemColor::Foreground))
                        .pos(pos),
                    position,
                )?;
            }
        }
        Ok(())
    }
}
