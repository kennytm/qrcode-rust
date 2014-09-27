#![unstable]

use std::default::Default;

//------------------------------------------------------------------------------
//{{{ QrResult

/// `QrError` encodes the error encountered when generating a QR code.
#[unstable]
#[deriving(Show, PartialEq, Eq)]
pub enum QrError {
    /// The data is too long to encode into a QR code for the given version.
    DataTooLong,

    /// The provided version / error correction level combination is invalid.
    InvalidVersion,

    /// Some characters in the data cannot be supported by the provided QR code
    /// version.
    UnsupportedCharacterSet,

    /// The provided ECI designator is invalid. A valid designator should be
    /// between 0 and 999999.
    InvalidEciDesignator,

    /// A character not belonging to the character set is found.
    InvalidCharacter,
}

/// `QrResult` is a convenient alias for a QR code generation result.
#[stable]
pub type QrResult<T> = Result<T, QrError>;

//}}}
//------------------------------------------------------------------------------
//{{{ Error correction level

/// The error correction level. It allows the original information be recovered
/// even if parts of the code is damaged.
#[deriving(Show, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[unstable]
pub enum ErrorCorrectionLevel {
    /// Low error correction. Allows up to 7% of wrong blocks.
    L = 0,

    /// Medium error correction (default). Allows up to 15% of wrong blocks.
    M = 1,

    /// "Quartile" error correction. Allows up to 25% of wrong blocks.
    Q = 2,

    /// High error correction. Allows up to 30% of wrong blocks.
    H = 3,
}

//}}}
//------------------------------------------------------------------------------
//{{{ Version

/// In QR code terminology, `Version` means the size of the generated image.
/// Larger version means the size of code is larger, and therefore can carry
/// more information.
///
/// The smallest version is `Version(1)` of size 21×21, and the largest is
/// `Version(40)` of size 177×177.
#[unstable]
#[deriving(Show, PartialEq, Eq, Copy, Clone)]
pub enum QrVersion {
    /// A normal QR code version. The parameter should be between 1 and 40.
    #[unstable]
    Version(i16),

    /// A Micro QR code version. The parameter should be between 1 and 4.
    MicroVersion(i16),
}

impl QrVersion {
    /// Get the number of "modules" on each size of the QR code, i.e. the width
    /// and height of the code.
    #[unstable]
    pub fn width(&self) -> i16 {
        match *self {
            Version(v) => v * 4 + 17,
            MicroVersion(v) => v * 2 + 9,
        }
    }

    /// Obtains an object from a hard-coded table.
    ///
    /// The table must be a 44×4 array. The outer array represents the content
    /// for each version. The first 40 entry corresponds to QR code versions 1
    /// to 40, and the last 4 corresponds to Micro QR code version 1 to 4. The
    /// inner array represents the content in each error correction level, in
    /// the order [L, M, Q, H].
    ///
    /// If the entry compares equal to the default value of T, this method
    /// returns `Err(InvalidVersion)`.
    pub fn fetch<T>(&self, ec_level: ErrorCorrectionLevel, table: &[[T, ..4]]) -> QrResult<T>
        where T: PartialEq + Default + Copy
    {
        match *self {
            Version(v @ 1...40) => Ok(table[v as uint - 1][ec_level as uint]),
            MicroVersion(v @ 1...4) => {
                let obj = table[v as uint + 39][ec_level as uint];
                if obj != Default::default() {
                    Ok(obj)
                } else {
                    Err(InvalidVersion)
                }
            }
            _ => Err(InvalidVersion)
        }
    }

    /// The number of bits needed to encode the mode indicator.
    #[unstable]
    pub fn mode_bits_count(&self) -> uint {
        match *self {
            MicroVersion(a) => (a - 1) as uint,
            _ => 4,
        }
    }

    /// Check whether is version refers to a Micro QR code.
    #[unstable]
    pub fn is_micro(&self) -> bool {
        match *self {
            Version(_) => false,
            MicroVersion(_) => true,
        }
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Mode indicator

/// The mode indicator, which specifies the character set of the encoded data.
#[unstable]
#[deriving(Show, PartialEq, Eq)]
pub enum Mode {
    /// The data contains only characters 0 to 9.
    Numeric,

    /// The data contains only uppercase letters (A–Z), numbers (0–9) and a few
    /// punctuations marks (space, `$`, `%`, `*`, `+`, `-`, `.`, `/`, `:`).
    Alphanumeric,

    /// The data contains arbitrary binary data.
    Byte,

    /// The data contains Shift-JIS-encoded double-byte text.
    Kanji,
}

impl Mode {
    /// Computes the number of bits needed to encode the data length.
    ///
    ///     use qrcode::types::{Version, Numeric};
    ///
    ///     assert_eq!(Numeric.length_bits_count(Version(1)), 10);
    ///
    /// This method will return `Err(UnsupportedCharacterSet)` if the is not
    /// supported in the given version.
    #[unstable]
    pub fn length_bits_count(&self, version: QrVersion) -> uint {
        match version {
            MicroVersion(a) => {
                let a = a as uint;
                match *self {
                    Numeric => 2 + a,
                    Alphanumeric | Byte => 1 + a,
                    Kanji => a,
                }
            }
            Version(1...9) => match *self {
                Numeric => 10,
                Alphanumeric => 9,
                Byte => 8,
                Kanji => 8,
            },
            Version(10...26) => match *self {
                Numeric => 12,
                Alphanumeric => 11,
                Byte => 16,
                Kanji => 10,
            },
            Version(_) => match *self {
                Numeric => 14,
                Alphanumeric => 13,
                Byte => 16,
                Kanji => 12,
            },
        }
    }

    /// Computes the number of bits needed to some data of a given raw length.
    ///
    ///     use qrcode::types::Numeric;
    ///
    ///     assert_eq!(Numeric.data_bits_count(7), 24);
    ///
    /// Note that in Kanji mode, the `raw_data_len` is the number of Kanjis,
    /// i.e. half the total size of bytes.
    #[unstable]
    pub fn data_bits_count(&self, raw_data_len: uint) -> uint {
        match *self {
            Numeric => (raw_data_len * 10 + 2) / 3,
            Alphanumeric => (raw_data_len * 11 + 1) / 2,
            Byte => raw_data_len * 8,
            Kanji => raw_data_len * 13,
        }
    }

    /// Find the lowest common mode which both modes are compatible with.
    ///
    ///     use qrcode::types::{Numeric, Kanji};
    ///
    ///     let a = Numeric;
    ///     let b = Kanji;
    ///     let c = a.max(b);
    ///     assert!(a <= c);
    ///     assert!(b <= c);
    ///
    pub fn max(&self, other: Mode) -> Mode {
        match self.partial_cmp(&other) {
            Some(Less) | Some(Equal) => other,
            Some(Greater) => *self,
            None => Byte,
        }
    }
}

impl PartialOrd for Mode {
    /// Defines a partial ordering between modes. If `a <= b`, then `b` contains
    /// a superset of all characters supported by `a`.
    fn partial_cmp(&self, other: &Mode) -> Option<Ordering> {
        match (*self, *other) {
            (Numeric, Alphanumeric) => Some(Less),
            (Alphanumeric, Numeric) => Some(Greater),
            (Numeric, Byte) => Some(Less),
            (Byte, Numeric) => Some(Greater),
            (Alphanumeric, Byte) => Some(Less),
            (Byte, Alphanumeric) => Some(Greater),
            (Kanji, Byte) => Some(Less),
            (Byte, Kanji) => Some(Greater),
            (a, b) if a == b => Some(Equal),
            _ => None,
        }
    }
}

#[cfg(test)]
mod mode_tests {
    use types::{Numeric, Alphanumeric, Byte, Kanji};

    #[test]
    fn test_mode_order() {
        assert!(Numeric < Alphanumeric);
        assert!(Byte > Kanji);
        assert!(!(Numeric < Kanji));
        assert!(!(Numeric >= Kanji));
    }

    #[test]
    fn test_max() {
        assert_eq!(Byte.max(Kanji), Byte);
        assert_eq!(Numeric.max(Alphanumeric), Alphanumeric);
        assert_eq!(Alphanumeric.max(Alphanumeric), Alphanumeric);
        assert_eq!(Numeric.max(Kanji), Byte);
        assert_eq!(Kanji.max(Numeric), Byte);
        assert_eq!(Alphanumeric.max(Numeric), Alphanumeric);
        assert_eq!(Kanji.max(Kanji), Kanji);
    }
}

//}}}

