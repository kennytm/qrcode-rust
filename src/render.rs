//! Render a QR code into image.

#![cfg(feature="image")]

use image::{Pixel, Rgb, Rgba, Luma, LumaA, Primitive, ImageBuffer};

/// A pixel which can support black and white colors.
pub trait BlankAndWhitePixel: Pixel {
    fn black_color() -> Self;
    fn white_color() -> Self;
}

impl<S: Primitive + 'static> BlankAndWhitePixel for Rgb<S> {
    fn black_color() -> Self {
        Rgb { data: [S::zero(); 3] }
    }

    fn white_color() -> Self {
        Rgb { data: [S::max_value(); 3] }
    }
}

impl<S: Primitive + 'static> BlankAndWhitePixel for Rgba<S> {
    fn black_color() -> Self {
        Rgba { data: [S::zero(), S::zero(), S::zero(), S::max_value()] }
    }

    fn white_color() -> Self {
        Rgba { data: [S::max_value(); 4] }
    }
}

impl<S: Primitive + 'static> BlankAndWhitePixel for Luma<S> {
    fn black_color() -> Self {
        Luma { data: [S::zero()] }
    }

    fn white_color() -> Self {
        Luma { data: [S::max_value()] }
    }
}

impl<S: Primitive + 'static> BlankAndWhitePixel for LumaA<S> {
    fn black_color() -> Self {
        LumaA { data: [S::zero(), S::max_value()] }
    }

    fn white_color() -> Self {
        LumaA { data: [S::max_value(); 2] }
    }
}

/// A QR code renderer. This is a builder type which converts a bool-vector into
/// an image.
pub struct Renderer<'a, P: BlankAndWhitePixel> {
    content: &'a [bool],
    modules_count: u32, // <- we call it `modules_count` here to avoid ambiguity of `width`.
    quiet_zone: u32,
    module_size: u32,

    dark_color: P,
    light_color: P,
    has_quiet_zone: bool,
}

impl<'a, P: BlankAndWhitePixel + 'static> Renderer<'a, P> {
    /// Creates a new renderer.
    pub fn new(content: &'a [bool], modules_count: usize, quiet_zone: u32) -> Renderer<'a, P> {
        assert!(modules_count * modules_count == content.len());
        Renderer {
            content: content,
            modules_count: modules_count as u32,
            quiet_zone: quiet_zone,
            module_size: 8,
            dark_color: P::black_color(),
            light_color: P::white_color(),
            has_quiet_zone: true,
        }
    }

    /// Sets color of a dark module. Default is opaque black.
    pub fn dark_color(&mut self, color: P) -> &mut Self {
        self.dark_color = color;
        self
    }

    /// Sets color of a light module. Default is opaque white.
    pub fn light_color(&mut self, color: P) -> &mut Self {
        self.light_color = color;
        self
    }

    /// Whether to include the quiet zone in the generated image.
    pub fn quiet_zone(&mut self, has_quiet_zone: bool) -> &mut Self {
        self.has_quiet_zone = has_quiet_zone;
        self
    }

    /// Sets the size of each module in pixels. Default is 8px.
    pub fn module_size(&mut self, size: u32) -> &mut Self {
        self.module_size = size;
        self
    }

    /// Sets the minimal total image width (and thus height) in pixels,
    /// including the quiet zone if applicable. The renderer will try to find
    /// the dimension as small as possible, such that each module in the QR code
    /// has uniform size (no distortion).
    ///
    /// For instance, a version 1 QR code has 19 modules across including the
    /// quiet zone. If we request an image of width ≥200px, we get that each
    /// module's size should be 11px, so the actual image size will be 209px.
    pub fn min_width(&mut self, width: u32) -> &mut Self {
        let quiet_zone = if self.has_quiet_zone { 2 } else { 0 } * self.quiet_zone;
        let width_in_modules = self.modules_count + quiet_zone;
        let module_size = (width + width_in_modules - 1) / width_in_modules;
        self.module_size(module_size)
    }

    /// Renders the QR code into an image.
    pub fn to_image(&self) -> ImageBuffer<P, Vec<P::Subpixel>> {
        let w = self.modules_count;
        let qz = if self.has_quiet_zone { self.quiet_zone } else { 0 };
        let width = w + 2 * qz;

        let ms = self.module_size;
        let real_width = width * ms;

        let mut image = ImageBuffer::new(real_width, real_width);
        let mut i = 0;
        for y in 0 .. width {
            for x in 0 .. width {
                let color = if qz <= x && x < w + qz && qz <= y && y < w + qz {
                    let c = if self.content[i] { self.dark_color } else { self.light_color };
                    i += 1;
                    c
                } else {
                    self.light_color
                };
                for yy in y * ms .. (y + 1) * ms {
                    for xx in x * ms .. (x + 1) * ms {
                        image.put_pixel(xx, yy, color);
                    }
                }
            }
        }

        image
    }

    /// Renders the QR code into String.
    ///
    /// # Examples
    ///
    /// Renders and prints QR Code in terminal:
    ///
    /// ```
    /// # extern crate qrcode;
    /// # extern crate image;
    ///
    /// # use qrcode::QrCode;
    /// # use image::Luma;
    ///
    /// # fn main() {
    ///    let code = QrCode::new(b"01234567").unwrap();
    ///    let s = code.render::<Luma<u8>>()
    ///        .to_string("\x1b[49m  \x1b[0m", "\x1b[7m  \x1b[0m");
    ///    println!("{}", s);
    /// # }
    /// ```
    pub fn to_string(&self, on_str: &str, off_str: &str) -> String {
        let w = self.modules_count;
        let qz = if self.has_quiet_zone { self.quiet_zone } else { 0 };
        let width = w + 2 * qz;

        let mut str = String::new();
        let mut i = 0;
        for y in 0..width {
            for x in 0..width {
                if qz <= x && x < w + qz && qz <= y && y < w + qz {
                    if self.content[i] {
                        str += on_str;
                    } else {
                        str += off_str;
                    };
                    i += 1;
                } else {
                    str += off_str;
                };
            }
            str.push('\n')
        }
        str
    }
}

#[cfg(test)]
mod render_tests {
    use render::Renderer;
    use image::{Luma, Rgba};

    #[test]
    fn test_render_luma8_unsized() {
        let image = Renderer::<Luma<u8>>::new(&[
            false, true, true,
            true, false, false,
            false, true, false,
        ], 3, 1).module_size(1).to_image();

        let expected = [
            255, 255, 255, 255, 255,
            255, 255,   0,   0, 255,
            255,   0, 255, 255, 255,
            255, 255,   0, 255, 255,
            255, 255, 255, 255, 255,
        ];
        assert_eq!(image.into_raw(), expected);
    }

    #[test]
    fn test_render_rgba_unsized() {
        let image = Renderer::<Rgba<u8>>::new(&[
            false, true,
            true, true,
        ], 2, 1).module_size(1).to_image();

        let expected: &[u8] = &[
            255,255,255,255, 255,255,255,255, 255,255,255,255, 255,255,255,255,
            255,255,255,255, 255,255,255,255,   0,  0,  0,255, 255,255,255,255,
            255,255,255,255,   0,  0,  0,255,   0,  0,  0,255, 255,255,255,255,
            255,255,255,255, 255,255,255,255, 255,255,255,255, 255,255,255,255,
        ];

        assert_eq!(image.into_raw(), expected);
    }

    #[test]
    fn test_render_resized() {
        let image = Renderer::<Luma<u8>>::new(&[
            true, false,
            false, true,
        ], 2, 1).min_width(10).to_image();

        let expected: &[u8] = &[
            255,255,255, 255,255,255, 255,255,255, 255,255,255,
            255,255,255, 255,255,255, 255,255,255, 255,255,255,
            255,255,255, 255,255,255, 255,255,255, 255,255,255,

            255,255,255,   0,  0,  0, 255,255,255, 255,255,255,
            255,255,255,   0,  0,  0, 255,255,255, 255,255,255,
            255,255,255,   0,  0,  0, 255,255,255, 255,255,255,

            255,255,255, 255,255,255,   0,  0,  0, 255,255,255,
            255,255,255, 255,255,255,   0,  0,  0, 255,255,255,
            255,255,255, 255,255,255,   0,  0,  0, 255,255,255,

            255,255,255, 255,255,255, 255,255,255, 255,255,255,
            255,255,255, 255,255,255, 255,255,255, 255,255,255,
            255,255,255, 255,255,255, 255,255,255, 255,255,255,
        ];

        assert_eq!(image.dimensions(), (12, 12));
        assert_eq!(image.into_raw(), expected);
    }
}


