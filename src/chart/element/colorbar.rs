// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::iter::{once, Once};

use plotters::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use plotters::element::{Drawable, PointCollection};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::FontDesc;

use crate::colormap::Colormap;

use crate::palette::{SystemColor, SystemPalette};

pub struct Colorbar<'a> {
    position: (i32, i32),
    size: (i32, i32),
    bounds: (f64, f64),
    precision: usize,
    unit: String,
    label_font: FontDesc<'a>,
    system_palette: SystemPalette,
    colormap: Colormap,
    n: i32,
}

impl<'a> Colorbar<'a> {
    #[allow(clippy::too_many_arguments)]
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
    type Borrow = &'a (i32, i32);
    type IntoIter = Once<&'a (i32, i32)>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.position)
    }
}

impl<'a, DB: DrawingBackend> Drawable<DB> for Colorbar<'a> {
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
        let step = height as f64 / (self.n as f64);
        for i in 0..self.n {
            let upper_left = (self.position.0, self.position.1 + (i as f64 * step) as i32);
            let bottom_right = (
                self.position.0 + 10,
                self.position.1 + ((i + 1) as f64 * step) as i32,
            );
            let value = self.bounds.0
                + (self.n - i - 1) as f64 * (self.bounds.1 - self.bounds.0) / (self.n - 1) as f64;
            let color = self.colormap.get_color(value);
            backend.draw_rect(upper_left, bottom_right, &color, true)?;

            if i % label_step == 0 {
                let pos = Pos::new(HPos::Left, VPos::Center);
                let position = (self.position.0 + 15, (upper_left.1 + bottom_right.1) / 2);
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
