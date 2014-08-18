//! QRCode encoder
//!
//! This crate provides a QR code and Micro QR code encoder for binary data.
//!
//!     use qrcode::QrCode;
//!
//!     let code = QrCode::new(b"Some content here.");
//!     match code {
//!         Err(err) => fail!("Failed to encode the QR code: {}", err),
//!         Ok(code) => {
//!             for y in range(0, code.width()) {
//!                 for x in range(0, code.width()) {
//!                     let color = if code[(x, y)] { "black" } else { "white" };
//!                     // render color at position (x, y)
//!                 }
//!             }
//!         }
//!     }
//!

#![unstable]

extern crate test;

use std::slice::CloneableVector;
pub use types::{QrResult, ErrorCorrectionLevel, L, M, Q, H, Version, MicroVersion};

pub mod types;
pub mod bits;
pub mod optimize;
pub mod ec;
pub mod canvas;

/// The encoded QR code symbol.
#[deriving(Clone)]
pub struct QrCode {
    content: Vec<bool>,
    version: Version,
    ec_level: ErrorCorrectionLevel,
    width: uint,
}

impl QrCode {
    /// Constructs a new QR code which automatically encodes the given data.
    ///
    /// This method uses the "medium" error correction level and automatically
    /// chooses the smallest QR code. Please use `.with_error_correction()` and
    /// `.with_version()` for finer adjustment.
    pub fn new(data: &[u8]) -> QrResult<QrCode> {
        QrCode::with_error_correction(data, M)
    }

    /// Constructs a new QR code which automatically encodes the given data at a
    /// specific error correction level.
    ///
    /// This method automatically chooses the smallest QR code.
    pub fn with_error_correction(data: &[u8], ec_level: ErrorCorrectionLevel) -> QrResult<QrCode> {
        let (encoded_data, version) = try!(bits::encode_auto(data, ec_level));
        QrCode::with_encoded_data(encoded_data.as_slice(), version, ec_level)
    }

    /// Constructs a new QR code for the given version and error correction
    /// level.
    pub fn with_version(data: &[u8],
                        version: Version,
                        ec_level: ErrorCorrectionLevel) -> QrResult<QrCode> {
        let encoded_data = try!(bits::encode(data, version, ec_level));
        QrCode::with_encoded_data(encoded_data.as_slice(), version, ec_level)
    }

    fn with_encoded_data(data: &[u8],
                         version: Version,
                         ec_level: ErrorCorrectionLevel) -> QrResult<QrCode> {
        let (encoded_data, ec_data) = try!(ec::construct_codewords(data, version, ec_level));
        let mut canvas = canvas::Canvas::new(version, ec_level);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(encoded_data.as_slice(), ec_data.as_slice());
        let canvas = canvas.apply_best_mask();
        Ok(QrCode {
            content: canvas.to_bools(),
            version: version,
            ec_level: ec_level,
            width: version.width() as uint,
        })
    }

    /// Gets the version of this QR code.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the error correction level of this QR code.
    pub fn error_correction_level(&self) -> ErrorCorrectionLevel {
        self.ec_level
    }

    /// Gets the number of modules per side, i.e. the width of this QR code.
    ///
    /// The width here does not contain the quiet zone paddings.
    pub fn width(&self) -> uint {
        self.width
    }

    /// Converts the QR code into a human-readable string. This is mainly for
    /// debugging only.
    pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
        let width = self.width;
        let mut k = 0;
        let mut res = String::with_capacity(width * (width + 1));
        for i in range(0, width) {
            res.push_char('\n');
            for _ in range(0, width) {
                res.push_char(if self.content[k] { on_char } else { off_char });
                k += 1;
            }
        }
        res
    }
}

impl Index<(uint, uint), bool> for QrCode {
    fn index<'a>(&'a self, &(x, y): &(uint, uint)) -> &'a bool {
        let index = y * self.width + x;
        self.content.index(&index)
    }
}

impl CloneableVector<bool> for QrCode {
    fn to_vec(&self) -> Vec<bool> {
        self.content.clone()
    }

    fn into_vec(self) -> Vec<bool> {
        self.content
    }
}

#[cfg(test)]
mod tests {
    use {QrCode, Version, MicroVersion, L, M};

    #[test]
    fn test_annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code = QrCode::with_version(b"01234567", Version(1), M).unwrap();
        assert_eq!(code.to_debug_str('#', '.').as_slice(), "\n\
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
        let code = QrCode::with_version(b"01234567", MicroVersion(2), L).unwrap();
        assert_eq!(code.to_debug_str('#', '.').as_slice(), "\n\
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

