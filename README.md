qrcode-rust
===========

[![Build status](https://travis-ci.org/kennytm/qrcode-rust.svg?branch=master)](https://travis-ci.org/kennytm/qrcode-rust)
[![Coverage Status](https://coveralls.io/repos/github/kennytm/qrcode-rust/badge.svg?branch=coveralls)](https://coveralls.io/github/kennytm/qrcode-rust?branch=coveralls)
[![crates.io](https://img.shields.io/crates/v/qrcode.svg)](https://crates.io/crates/qrcode)
[![MIT / Apache 2.0](https://img.shields.io/badge/license-MIT%20%2f%20Apache%202.0-blue.svg)](./LICENSE-APACHE.txt)

QR code and Micro QR code encoder in Rust. [Documentation](https://docs.rs/qrcode).

Cargo.toml
----------

```toml
[dependencies]
qrcode = "0.4"
```

The default settings will depend on the `image` crate. If you don't need image generation capability, disable the `default-features`:

```toml
[dependencies]
qrcode = { version = "0.4", default-features = false }
```

Example
-------

This code:

```rust
extern crate qrcode;
extern crate image;

use qrcode::QrCode;
use image::GrayImage;

fn main() {
    // Encode some data into bits.
    let code = QrCode::new(b"01234567").unwrap();

    // Render the bits into an image.
    let image: GrayImage = code.render().to_image();

    // Save the image.
    image.save("/tmp/qrcode.png").unwrap();
}
```

Generates this image:

![Output](src/test_annex_i_qr_as_image.png)

