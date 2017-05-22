extern crate qrcode;

use std::env;

const SPACE: char = ' '; //'　';

pub fn main() {
    let arg = env::args().nth(1).unwrap();
    let code = qrcode::QrCode::new(arg.as_bytes()).unwrap();

    print!("\n\n\n\n\n{}{}{}{}{}", SPACE, SPACE, SPACE, SPACE, SPACE);

    for y in 0 .. code.width() {
        for x in 0 .. code.width() {
            let block = code[(x, y)].select('█', SPACE);
            print!("{}{}", block, block);
        }
        print!("\n{}{}{}{}{}", SPACE, SPACE, SPACE, SPACE, SPACE);
    }

    println!("\n\n\n\n");
}

