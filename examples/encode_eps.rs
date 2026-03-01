use qrcode::render::eps;
use qrcode::{EcLevel, QrCode, Version};

fn main() {
    let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(eps::Color([0.5, 0.0, 0.0]))
        .light_color(eps::Color([1.0, 1.0, 0.5]))
        .build();
    println!("{image}");
}
