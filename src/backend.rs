// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::path::Path;

use plotters::drawing::backend::DrawingErrorKind;
use plotters::drawing::backend::BackendCoord;
use plotters::drawing::DrawingBackend;
use plotters::drawing::BitMapBackend;
use plotters::style::RGBAColor;

use crate::error::DashboardError;

pub enum BackendTypeWrapper<'a> {
    FrameBuffer(&'a Path, &'a mut [u8]),
    File(BitMapBackend<'a>),
}

pub struct OtherBackendType<'a> {
    backend: BackendTypeWrapper<'a>,
    resolution: (u32, u32),
}

impl <'a> OtherBackendType<'a> {
    pub fn new_from_path(path: &'a Path, resolution: (u32, u32)) -> Self {
        info!("Creating bitmap at {}", path.display());
        OtherBackendType {
            backend: BackendTypeWrapper::File(BitMapBackend::new(path, resolution)),
            resolution,
        }
    }
    pub fn new_from_frame_buffer(device: &'a Path, buffer: &'a mut [u8], resolution: (u32, u32)) -> Self {
        OtherBackendType {
            backend: BackendTypeWrapper::FrameBuffer(device, buffer),
            resolution,
        }
    }
}

impl <'a> DrawingBackend for OtherBackendType<'a> {
    type ErrorType = DashboardError;

    fn get_size(&self) -> (u32, u32) {
        self.resolution
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match &mut self.backend {
            &mut BackendTypeWrapper::File(ref mut backend) => backend.ensure_prepared(),
            BackendTypeWrapper::FrameBuffer(_, _) => Ok(())
        }.map_err(|_| DrawingErrorKind::DrawingError(DashboardError::Unknown))
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match &mut self.backend {
            &mut BackendTypeWrapper::File(ref mut backend) => backend.present().map_err(|_| DrawingErrorKind::DrawingError(DashboardError::Unknown))?,
            BackendTypeWrapper::FrameBuffer(ref device, ref buffer) => {
                crate::framebuffer::display_image(device, &buffer, self.resolution.0, self.resolution.1).map_err(|_| DrawingErrorKind::DrawingError(DashboardError::Unknown))?;
            }
        }

        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match &mut self.backend {
            &mut BackendTypeWrapper::File(ref mut backend) => backend.draw_pixel(point, color),
            BackendTypeWrapper::FrameBuffer(_, ref mut buffer) => {
                let mut backend = BitMapBackend::with_buffer(buffer, self.resolution);
                backend.draw_pixel(point, color)
            }
        }.map_err(|k| {
            info!("k: {}", k);
            DrawingErrorKind::DrawingError(DashboardError::Unknown)
        })
    }
}
