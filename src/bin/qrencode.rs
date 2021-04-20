use std::env;

pub fn main() {
    if env::args().len() == 2 {
        let arg = env::args().nth(1).unwrap();
        let code = qrcode::QrCode::new(arg.as_bytes()).unwrap();

        println!("{}", code.render().dark_color("\x1b[7m  \x1b[0m").light_color("\x1b[49m  \x1b[0m").build());
    } else {
        println!("Usage: {} INPUT_TEXT", env::args().nth(0).unwrap());
    }
}
