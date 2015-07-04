//! QRCode encoder
//!
//! This crate provides a QR code and Micro QR code encoder for binary data.
//!
//!     use qrcode::QrCode;
//!
//!     let code = QrCode::new(b"Some content here.");
//!     match code {
//!         Err(err) => panic!("Failed to encode the QR code: {:?}", err),
//!         Ok(code) => {
//!             for y in 0 .. code.width() {
//!                 for x in 0 .. code.width() {
//!                     let color = if code[(x, y)] { "black" } else { "white" };
//!                     // render color at position (x, y)
//!                 }
//!             }
//!         }
//!     }
//!

#![cfg_attr(feature="bench", feature(test))] // Unstable libraries

#[cfg(feature="bench")]
extern crate test;
extern crate num;

use std::ops::Index;

pub use types::{QrResult, EcLevel, Version};

pub mod types;
pub mod bits;
pub mod optimize;
pub mod ec;
pub mod canvas;

/// The encoded QR code symbol.
#[derive(Clone)]
pub struct QrCode {
    content: Vec<bool>,
    version: Version,
    ec_level: EcLevel,
    width: usize,
}

impl QrCode {
    /// Constructs a new QR code which automatically encodes the given data.
    ///
    /// This method uses the "medium" error correction level and automatically
    /// chooses the smallest QR code.
    ///
    ///     use qrcode::QrCode;
    ///
    ///     let code = QrCode::new(b"Some data").unwrap();
    ///
    pub fn new(data: &[u8]) -> QrResult<QrCode> {
        QrCode::with_error_correction_level(data, EcLevel::M)
    }

    /// Constructs a new QR code which automatically encodes the given data at a
    /// specific error correction level.
    ///
    /// This method automatically chooses the smallest QR code.
    ///
    ///     use qrcode::{QrCode, EcLevel};
    ///
    ///     let code = QrCode::with_error_correction_level(b"Some data", EcLevel::H).unwrap();
    ///
    pub fn with_error_correction_level(data: &[u8], ec_level: EcLevel) -> QrResult<QrCode> {
        let bits = try!(bits::encode_auto(data, ec_level));
        QrCode::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code for the given version and error correction
    /// level.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let code = QrCode::with_version(b"Some data", Version::Normal(5), EcLevel::M).unwrap();
    ///
    /// This method can also be used to generate Micro QR code.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let micro_code = QrCode::with_version(b"123", Version::Micro(1), EcLevel::L).unwrap();
    ///
    pub fn with_version(data: &[u8], version: Version, ec_level: EcLevel) -> QrResult<QrCode> {
        let mut bits = bits::Bits::new(version);
        try!(bits.push_optimal_data(data));
        try!(bits.push_terminator(ec_level));
        QrCode::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code with encoded bits.
    ///
    /// Use this method only if there are very special need to manipulate the
    /// raw bits before encoding. Some examples are:
    ///
    /// * Encode data using specific character set with ECI
    /// * Use the FNC1 modes
    /// * Avoid the optimal segmentation algorithm
    ///
    /// See the `Bits` structure for detail.
    ///
    ///     #![allow(unused_must_use)]
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///     use qrcode::bits::Bits;
    ///
    ///     let mut bits = Bits::new(Version::Normal(1));
    ///     bits.push_eci_designator(9);
    ///     bits.push_byte_data(b"\xca\xfe\xe4\xe9\xea\xe1\xf2 QR");
    ///     bits.push_terminator(EcLevel::L);
    ///     let qrcode = QrCode::with_bits(bits, EcLevel::L);
    ///
    pub fn with_bits(bits: bits::Bits, ec_level: EcLevel) -> QrResult<QrCode> {
        let version = bits.version();
        let data = bits.into_bytes();
        let (encoded_data, ec_data) = try!(ec::construct_codewords(&*data, version, ec_level));
        let mut canvas = canvas::Canvas::new(version, ec_level);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&*encoded_data, &*ec_data);
        let canvas = canvas.apply_best_mask();
        Ok(QrCode {
            content: canvas.to_bools(),
            version: version,
            ec_level: ec_level,
            width: version.width() as usize,
        })
    }

    /// Gets the version of this QR code.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the error correction level of this QR code.
    pub fn error_correction_level(&self) -> EcLevel {
        self.ec_level
    }

    /// Gets the number of modules per side, i.e. the width of this QR code.
    ///
    /// The width here does not contain the quiet zone paddings.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Gets the maximum number of allowed erratic modules can be introduced
    /// before the data becomes corrupted. Note that errors should not be
    /// introduced to functional modules.
    pub fn max_allowed_errors(&self) -> usize {
        ec::max_allowed_errors(self.version, self.ec_level).unwrap()
    }

    /// Checks whether a module at coordinate (x, y) is a functional module or
    /// not.
    pub fn is_functional(&self, x: usize, y: usize) -> bool {
        canvas::is_functional(self.version, self.version.width(), x as i16, y as i16)
    }

    /// Converts the QR code into a human-readable string. This is mainly for
    /// debugging only.
    pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
        let width = self.width;
        let mut k = 0;
        let mut res = String::with_capacity(width * (width + 1));
        for _ in 0 .. width {
            res.push('\n');
            for _ in 0 .. width {
                res.push(if self.content[k] { on_char } else { off_char });
                k += 1;
            }
        }
        res
    }

    /// Converts the QR code to a vector of booleans. Each entry represents the
    /// color of the module, with "true" means dark and "false" means light.
    pub fn to_vec(&self) -> Vec<bool> {
        self.content.clone()
    }

    /// Converts the QR code to a vector of booleans. Each entry represents the
    /// color of the module, with "true" means dark and "false" means light.
    pub fn into_vec(self) -> Vec<bool> {
        self.content.clone()
    }
}

impl Index<(usize, usize)> for QrCode {
    type Output = bool;

    fn index(&self, (x, y): (usize, usize)) -> &bool {
        let index = y * self.width + x;
        &self.content[index]
    }
}

#[cfg(test)]
mod tests {
    use {QrCode, Version, EcLevel};

    #[test]
    fn test_annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code = QrCode::with_version(b"01234567", Version::Normal(1), EcLevel::M).unwrap();
        assert_eq!(&*code.to_debug_str('#', '.'), "\n\
                    #######..#.##.#######\n\
                    #.....#..####.#.....#\n\
                    #.###.#.#.....#.###.#\n\
                    #.###.#.##....#.###.#\n\
                    #.###.#.#.###.#.###.#\n\
                    #.....#.#...#.#.....#\n\
                    #######.#.#.#.#######\n\
                    ........#..##........\n\
                    #.#####..#..#.#####..\n\
                    ...#.#.##.#.#..#.##..\n\
                    ..#...##.#.#.#..#####\n\
                    ....#....#.....####..\n\
                    ...######..#.#..#....\n\
                    ........#.#####..##..\n\
                    #######..##.#.##.....\n\
                    #.....#.#.#####...#.#\n\
                    #.###.#.#...#..#.##..\n\
                    #.###.#.##..#..#.....\n\
                    #.###.#.#.##.#..#.#..\n\
                    #.....#........##.##.\n\
                    #######.####.#..#.#..");
    }

    #[test]
    fn test_annex_i_micro_qr() {
        let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
        assert_eq!(&*code.to_debug_str('#', '.'), "\n\
                    #######.#.#.#\n\
                    #.....#.###.#\n\
                    #.###.#..##.#\n\
                    #.###.#..####\n\
                    #.###.#.###..\n\
                    #.....#.#...#\n\
                    #######..####\n\
                    .........##..\n\
                    ##.#....#...#\n\
                    .##.#.#.#.#.#\n\
                    ###..#######.\n\
                    ...#.#....##.\n\
                    ###.#..##.###");
    }
}

// Copyright 2014 Kenny Chan
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

