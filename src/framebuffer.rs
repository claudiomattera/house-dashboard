// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::path::Path;

use framebuffer::{Framebuffer, KdMode};

#[allow(dead_code)]
pub fn display_image(
            fb_device: &Path,
            image: &[u8],
            image_width: u32,
            image_height: u32,
        ) -> Result<(), Box<dyn std::error::Error>> {

    info!("Opening framebuffer {}", fb_device.display());
    let mut framebuffer = Framebuffer::new(fb_device.to_str().expect("Invalid path"))?;
    let width = framebuffer.var_screen_info.xres;
    let height = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytes_per_pixel = framebuffer.var_screen_info.bits_per_pixel / 8;

    let red_length = framebuffer.var_screen_info.red.length;
    let green_length = framebuffer.var_screen_info.green.length;
    let blue_length = framebuffer.var_screen_info.blue.length;

    let red_offset = framebuffer.var_screen_info.red.offset;
    let green_offset = framebuffer.var_screen_info.green.offset;
    let blue_offset = framebuffer.var_screen_info.blue.offset;

    let red_mask = (u32::pow(2, red_length) - 1) << red_offset;
    let green_mask = (u32::pow(2, green_length) - 1) << green_offset;
    let blue_mask = (u32::pow(2, blue_length) - 1) << blue_offset;

    debug!("width: {}", width);
    debug!("height: {}", height);
    debug!("line length: {}", line_length);
    debug!("bytes per pixel: {}", bytes_per_pixel);

    debug!("Red mask:   {:032b}", red_mask);
    debug!("Green mask: {:032b}", green_mask);
    debug!("Blue mask:  {:032b}", blue_mask);

    let mut frame = vec![0u8; (line_length * height) as usize];

    debug!("width: {}", image_width);
    debug!("height: {}", image_height);

    debug!("Disabling text mode in current tty");
    match Framebuffer::set_kd_mode(KdMode::Graphics) {
        Ok(_) => {}
        Err(error) => {
            warn!("Error disabling text mode in current tty: {:?}", error);
        }
    }

    debug!("Populating frame");
    for x in 0..image_width {
        for y in 0..height {
            let input_index = (y + x * bytes_per_pixel) as usize;
            let red = image[input_index];
            let green = image[input_index + 1];
            let blue = image[input_index + 2];

            let pixel = pixel_to_pixel(
                (red, green, blue),
                (red_length, green_length, blue_length),
                (red_offset, green_offset, blue_offset),
            );

            let output_index = (y * line_length + x * bytes_per_pixel) as usize;
            for i in 0..bytes_per_pixel {
                frame[output_index + (i as usize)] = ((pixel >> (8 * i)) & 0xff) as u8;
            }
        }
    }

    debug!("Writing frame to framebuffer");
    framebuffer.write_frame(&frame);

    debug!("Re-enabling text mode in current tty");
    match Framebuffer::set_kd_mode(KdMode::Text) {
        Ok(_) => {}
        Err(error) => {
            warn!("Error reenabling text mode in current tty: {:?}", error);
        }
    }

    Ok(())
}

fn pixel_to_pixel(
            original: (u8, u8, u8),
            length: (u32, u32, u32),
            offset: (u32, u32, u32),
        ) -> u32 {
    let red: u32 = (original.0 as u32) * (1 << length.0) / 256;
    let green: u32 = (original.1 as u32) * (1 << length.1) / 256;
    let blue: u32 = (original.2 as u32) * (1 << length.2) / 256;
    red * (1 << offset.0) + green * (1 << offset.1) + blue * (1 << offset.2)
}

#[cfg(test)]
mod tests {
    use crate::framebuffer::*;

    #[test]
    fn white_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0xff, 0xff, 0xff), (5, 6, 5), (11, 5, 0));
        let expected = 0xffff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn black_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0x00, 0x00, 0x00), (5, 6, 5), (11, 5, 0));
        let expected = 0x0000;
        assert_eq!(actual, expected);
    }

    #[test]
    fn red_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0xff, 0x00, 0x00), (5, 6, 5), (11, 5, 0));
        let expected = 0xf800;
        assert_eq!(actual, expected);
    }

    #[test]
    fn green_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0x00, 0xff, 0x00), (5, 6, 5), (11, 5, 0));
        let expected = 0x07e0;
        assert_eq!(actual, expected);
    }

    #[test]
    fn blue_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0x00, 0x00, 0xff), (5, 6, 5), (11, 5, 0));
        let expected = 0x001f;
        assert_eq!(actual, expected);
    }

    #[test]
    fn fuchsia_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0xff, 0x00, 0xff), (5, 6, 5), (11, 5, 0));
        let expected = 0xf81f;
        assert_eq!(actual, expected);
    }

    #[test]
    fn aqua_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0x00, 0xff, 0xff), (5, 6, 5), (11, 5, 0));
        let expected = 0x07ff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn yellow_pixel_to_pixel_16() {
        let actual = pixel_to_pixel((0xff, 0xff, 0x00), (5, 6, 5), (11, 5, 0));
        let expected = 0xffe0;
        assert_eq!(actual, expected);
    }

    #[test]
    fn white_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0xff, 0xff, 0xff), (8, 8, 8), (16, 8, 0));
        let expected = 0x00ff_ffff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn black_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0x00, 0x00, 0x00), (8, 8, 8), (16, 8, 0));
        let expected = 0x0000_0000;
        assert_eq!(actual, expected);
    }

    #[test]
    fn red_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0xff, 0x00, 0x00), (8, 8, 8), (16, 8, 0));
        let expected = 0x00ff_0000;
        assert_eq!(actual, expected);
    }

    #[test]
    fn green_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0x00, 0xff, 0x00), (8, 8, 8), (16, 8, 0));
        let expected = 0x0000_ff00;
        assert_eq!(actual, expected);
    }

    #[test]
    fn blue_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0x00, 0x00, 0xff), (8, 8, 8), (16, 8, 0));
        let expected = 0x0000_00ff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn fuchsia_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0xff, 0x00, 0xff), (8, 8, 8), (16, 8, 0));
        let expected = 0x00ff_00ff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn aqua_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0x00, 0xff, 0xff), (8, 8, 8), (16, 8, 0));
        let expected = 0x0000_ffff;
        assert_eq!(actual, expected);
    }

    #[test]
    fn yellow_pixel_to_pixel_32() {
        let actual = pixel_to_pixel((0xff, 0xff, 0x00), (8, 8, 8), (16, 8, 0));
        let expected = 0x00ff_ff00;
        assert_eq!(actual, expected);
    }
}
