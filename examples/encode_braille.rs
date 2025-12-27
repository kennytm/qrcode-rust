use qrcode::render::braille;
use qrcode::QrCode;

fn main() {
    let code = QrCode::new(b"Hello").unwrap();
    let string = code.render::<braille::BraillePixel>().quiet_zone(false).build();
    println!("{string}");
}
