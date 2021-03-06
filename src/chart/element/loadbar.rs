// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::iter::{once, Once};

use plotters::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use plotters::element::{Drawable, PointCollection};

use crate::colormap::Colormap;

use crate::palette::{SystemColor, SystemPalette};

pub struct Loadbar<'a> {
    position: (i32, i32),
    size: (i32, i32),
    max: f64,
    value: f64,
    system_palette: &'a SystemPalette,
    colormap: &'a Colormap,
    n: i32,
}

impl<'a> Loadbar<'a> {
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
    type Borrow = &'a (i32, i32);
    type IntoIter = Once<&'a (i32, i32)>;
    fn point_iter(self) -> Self::IntoIter {
        once(&self.position)
    }
}

impl<'a, DB: DrawingBackend> Drawable<DB> for Loadbar<'a> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = pos.next() {
            let upper_left = (x - self.size.0 / 2 - 1, y - self.size.1 / 2 - 1);
            let bottom_right = (x + self.size.0 / 2 + 1, y + self.size.1 / 2 + 1);
            backend.draw_rect(
                upper_left,
                bottom_right,
                &self.system_palette.pick(SystemColor::LightForeground),
                false,
            )?;

            let width = self.size.0;

            let step = width as f64 / (self.n as f64);
            let stop = ((self.n as f64 * self.value) / self.max) as i32;
            for i in 0..self.n {
                if i >= stop {
                    break;
                }
                let upper_left = (
                    x - self.size.0 / 2 + (i as f64 * step) as i32,
                    y - self.size.1 / 2,
                );
                let bottom_right = (
                    x - self.size.0 / 2 + ((i + 1) as f64 * step) as i32,
                    y + self.size.1 / 2,
                );
                let value = i as f64 * self.max / self.n as f64;
                let color = self.colormap.get_color(value);
                backend.draw_rect(upper_left, bottom_right, &color, true)?;
            }
        }
        Ok(())
    }
}
