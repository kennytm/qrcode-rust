//! UTF-8 rendering, with 2 pixels per symbol.

use crate::render::{Canvas as RenderCanvas, Pixel, Color};

impl Pixel for u8 {
    type Image = String;
    type Canvas = Canvas;
    fn default_color(color: Color) -> u8 {if color == Color::Dark {1} else {0}}
    fn default_unit_size() -> (u32, u32) { (1, 1) }
}

pub struct Canvas {
    canvas: Vec<u8>,
    width: u32,
    dark_pixel: u8
}

impl RenderCanvas for Canvas {

    type Pixel = u8;
    type Image = String;

    
    fn new(width: u32, height: u32, dark_pixel: u8, light_pixel: u8) -> Self { 
        let a = vec![light_pixel; (width * height) as usize];
        Canvas {
            width: width,
            canvas: a,
            dark_pixel: dark_pixel
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
                }.iter()
                // Mapping those 2-bit numbers to corresponding pixels.
                .map(|sym| [" ","\u{2584}","\u{2580}","\u{2588}"][*sym as usize])
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
    let image: String = Renderer::<u8>::new(colors, 2, 1).build();
    
    assert_eq!(&image, " ▄  \n  ▀ ");

    let image2 = Renderer::<u8>::new(colors, 2, 1).module_dimensions(2, 2).build();

    assert_eq!(&image2, "        \n  ██    \n    ██  \n        ");
}
