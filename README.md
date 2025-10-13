qrcode-rust
===========

[![Build status](https://github.com/kennytm/qrcode-rust/workflows/Rust/badge.svg)](https://github.com/kennytm/qrcode-rust/actions?query=workflow%3ARust)
[![crates.io](https://img.shields.io/crates/v/qrcode.svg)](https://crates.io/crates/qrcode)
[![MIT OR Apache 2.0](https://img.shields.io/badge/license-MIT%20%2f%20Apache%202.0-blue.svg)](./LICENSE-APACHE.txt)

QR code, Micro QR code and rMQR code encoder in Rust. [Documentation](https://docs.rs/qrcode).

Cargo.toml
----------

```toml
[dependencies]
qrcode = "0.14.1"
```

The default settings will depend on the `image` crate. If you don't need image generation capability, disable the `default-features`:

```toml
[dependencies]
qrcode = { version = "0.14.1", default-features = false, features = ["std"] }
```

Example
-------

### Image generation

```rust
use qrcode::QrCode;
use image::Luma;

fn main() {
    // Encode some data into bits.
    let code = QrCode::new(b"01234567").unwrap();

    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    image.save("/tmp/qrcode.png").unwrap();
}
```

Generates this image:

![Output](src/test_annex_i_qr_as_image.png)

### String generation

```rust
use qrcode::QrCode;

fn main() {
    let code = QrCode::new(b"Hello").unwrap();
    let string = code.render::<char>()
        .dark_color('#')
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{string}");
}
```

Generates this output:

```none
##############    ########  ##############
##          ##          ##  ##          ##
##  ######  ##  ##  ##  ##  ##  ######  ##
##  ######  ##  ##  ##      ##  ######  ##
##  ######  ##  ####    ##  ##  ######  ##
##          ##  ####  ##    ##          ##
##############  ##  ##  ##  ##############
                ##  ##
##  ##########    ##  ##    ##########
      ##        ##    ########    ####  ##
    ##########    ####  ##  ####  ######
    ##    ##  ####  ##########    ####
  ######    ##########  ##    ##        ##
                ##      ##    ##  ##
##############    ##  ##  ##    ##  ####
##          ##  ##  ##        ##########
##  ######  ##  ##    ##  ##    ##    ##
##  ######  ##  ####  ##########  ##
##  ######  ##  ####    ##  ####    ##
##          ##    ##  ########  ######
##############  ####    ##      ##    ##
```

### SVG generation

```rust
use qrcode::{QrCode, Version, EcLevel};
use qrcode::render::svg;

fn main() {
    let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
    let image = code.render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#800000"))
        .light_color(svg::Color("#ffff80"))
        .build();
    println!("{image}");
}
```

Generates this SVG:

[![Output](src/test_annex_i_micro_qr_as_svg.svg)](src/test_annex_i_micro_qr_as_svg.svg)

### Unicode string generation

```rust
use qrcode::QrCode;
use qrcode::render::unicode;

fn main() {
    let code = QrCode::new("mow mow").unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{image}");
}
```

Generates this output:

```text
█████████████████████████████
█████████████████████████████
████ ▄▄▄▄▄ █ ▀▀▀▄█ ▄▄▄▄▄ ████
████ █   █ █▀ ▀ ▀█ █   █ ████
████ █▄▄▄█ ██▄  ▀█ █▄▄▄█ ████
████▄▄▄▄▄▄▄█ ▀▄▀ █▄▄▄▄▄▄▄████
████▄▀ ▄▀ ▄ █▄█  ▀ ▀█ █▄ ████
████▄██▄▄▀▄▄▀█▄ ██▀▀█▀▄▄▄████
█████▄▄▄█▄▄█  ▀▀▄█▀▀▀▄█▄▄████
████ ▄▄▄▄▄ █   ▄▄██▄ ▄ ▀▀████
████ █   █ █▀▄▄▀▄▄ ▄▄▄▄ ▄████
████ █▄▄▄█ █▄  █▄▀▄▀██▄█▀████
████▄▄▄▄▄▄▄█▄████▄█▄██▄██████
█████████████████████████████
▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
```

### PIC generation

```rust
use qrcode::render::pic;
use qrcode::QrCode;

fn main() {
    let code = QrCode::new(b"01234567").unwrap();
    let image = code
        .render::<pic::Color>()
        .min_dimensions(1, 1)
        .build();
    println!("{image}");
}
```

Generates [PIC](https://en.wikipedia.org/wiki/PIC_(markup_language))
output that renders as follows:

```pic
maxpswid=29;maxpsht=29;movewid=0;moveht=1;boxwid=1;boxht=1
define p { box wid $3 ht $4 fill 1 thickness 0.1 with .nw at $1,-$2 }
box wid maxpswid ht maxpsht with .nw at 0,0
p(4,4,1,1)
p(5,4,1,1)
p(6,4,1,1)
p(7,4,1,1)
p(8,4,1,1)
p(9,4,1,1)
…
```
See [`test_annex_i_micro_qr_as_pic.pic`](src/test_annex_i_micro_qr_as_pic.pic) for a full example.

### EPS generation

```rust
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
```

Generates [EPS](https://en.wikipedia.org/wiki/Encapsulated_PostScript)
output that renders as follows:

```postscript
%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 0 204 204
%%Pages: 1
%%EndComments
gsave
1 1 0.5 setrgbcolor
0 0 204 204 rectfill
grestore
0.5 0 0 setrgbcolor
24 180 12 12 rectfill
36 180 12 12 rectfill
48 180 12 12 rectfill
60 180 12 12 rectfill
72 180 12 12 rectfill
84 180 12 12 rectfill
…
```
See [`test_annex_i_micro_qr_as_eps.eps`](src/test_annex_i_micro_qr_as_eps.eps) for a full example.

License
-------

Copyright (c) 2016 kennytm

This crate is licensed under either of the
[MIT License](https://spdx.org/licenses/MIT.html) or the
[Apache License 2.0](https://spdx.org/licenses/Apache-2.0.html) at your option.

The rMQR code encoder is based on the
[`qrqrpar`](https://crates.io/crates/qrqrpar) crate, which is a fork of this
crate. It is licensed under the
[BSD 3-Clause "New" or "Revised" License](https://spdx.org/licenses/BSD-3-Clause.html).
