//! UTF-8 rendering, with 2 pixels per symbol.

use crate::render::{Canvas as RenderCanvas, Pixel, Color};

const CODEPAGE: [&str; 4] = [" ","\u{2584}","\u{2580}","\u{2588}"];

#[derive(Copy, Clone, PartialEq)]
pub enum Unicode1x2 {
    Dark, Light
}

impl Pixel for Unicode1x2 {
    type Image = String;
    type Canvas = Canvas;
    fn default_color(color: Color) -> Unicode1x2 { color.select(Unicode1x2::Dark, Unicode1x2::Light) }
    fn default_unit_size() -> (u32, u32) { (1, 1) }
}

impl Unicode1x2 {
    fn value(&self) -> u8 {
        match self {
            Unicode1x2::Dark => {1}
            Unicode1x2::Light => {0}
        }
    }
    #[doc(hidden)]
    fn parse_2_bits(sym: &u8) -> &'static str {
        CODEPAGE[*sym as usize]
    }
}

pub struct Canvas {
    canvas: Vec<u8>,
    width: u32,
    dark_pixel: u8
}

impl RenderCanvas for Canvas {

    type Pixel = Unicode1x2;
    type Image = String;

    
    fn new(width: u32, height: u32, dark_pixel: Unicode1x2, light_pixel: Unicode1x2) -> Self { 
        let a = vec![light_pixel.value(); (width * height) as usize];
        Canvas {
            width: width,
            canvas: a,
            dark_pixel: dark_pixel.value()
        }
    }

    fn draw_dark_pixel(&mut self, x: u32, y: u32) { 
        self.canvas[(x + y * self.width) as usize] = self.dark_pixel;
    }

    fn into_image(self) -> String { 
        self.canvas
            // Chopping array into 1-line sized fragments
            .chunks_exact(self.width as usize)
            .collect::<Vec<&[u8]>>()
            // And then glueing every 2 lines.
            .chunks(2)
            .map(|rows|
                {
                    // Then zipping those 2 lines together into a single 2-bit number list.
                    if rows.len() == 2 {
                        rows[0].iter().zip(rows[1]).map(|(top,bot)| (top * 2 + bot)).collect::<Vec<u8>>()
                    } else {
                        rows[0].iter().map(|top| (top * 2)).collect::<Vec<u8>>()
                    }
                }
                .iter()
                // Mapping those 2-bit numbers to corresponding pixels.
                .map(Unicode1x2::parse_2_bits)
                .collect::<Vec<&str>>()
                .concat()
            )
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[test]
fn test_render_to_utf8_string() {
    use crate::render::Renderer;
    let colors = &[Color::Dark, Color::Light, Color::Light, Color::Dark];
    let image: String = Renderer::<Unicode1x2>::new(colors, 2, 1).build();
    
    assert_eq!(&image, " ▄  \n  ▀ ");

    let image2 = Renderer::<Unicode1x2>::new(colors, 2, 1).module_dimensions(2, 2).build();

    assert_eq!(&image2, "        \n  ██    \n    ██  \n        ");
}

#[test]
fn integration_render_utf8_1x2() {
    use crate::{QrCode, Version, EcLevel};
    use crate::render::utf8::Unicode1x2;

    let code = QrCode::with_version(b"12345678", Version::Micro(2), EcLevel::L).unwrap();
    let image = code.render::<Unicode1x2>()
        .dark_color(Unicode1x2::Light)
        .light_color(Unicode1x2::Dark)
        .module_dimensions(1, 1)
        .build();
    assert_eq!("█████████████████\n██ ▄▄▄▄▄ █▄▀▄█▄██\n██ █   █ █   █ ██\n██ █▄▄▄█ █▄▄██▀██\n██▄▄▄▄▄▄▄█▄▄▄▀ ██\n██▄ ▀ ▀ ▀▄▄  ████\n██▄▄▀▄█ ▀▀▀ ▀▄▄██\n██▄▄▄█▄▄█▄██▄█▄██\n▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀", image);

}