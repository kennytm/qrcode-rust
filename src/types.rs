//! The `types` module contains types associated with the functional elements of
//! a QR code.

use crate::cast::As;
use core::cmp::{Ordering, PartialOrd};
use core::default::Default;
use core::fmt::{Display, Error, Formatter};
use core::ops::Not;

//------------------------------------------------------------------------------
//{{{ QrResult

/// `QrError` encodes the error encountered when generating a QR code.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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

impl Display for QrError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let msg = match *self {
            Self::DataTooLong => "data too long",
            Self::InvalidVersion => "invalid version",
            Self::UnsupportedCharacterSet => "unsupported character set",
            Self::InvalidEciDesignator => "invalid ECI designator",
            Self::InvalidCharacter => "invalid character",
        };
        fmt.write_str(msg)
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for QrError {}

/// `QrResult` is a convenient alias for a QR code generation result.
pub type QrResult<T> = Result<T, QrError>;

//}}}
//------------------------------------------------------------------------------
//{{{ Color

/// The color of a module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    /// The module is light colored.
    Light,
    /// The module is dark colored.
    Dark,
}

impl Color {
    /// Selects a value according to color of the module. Equivalent to
    /// `if self != Color::Light { dark } else { light }`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qrcode::types::Color;
    /// assert_eq!(Color::Light.select(1, 0), 0);
    /// assert_eq!(Color::Dark.select("black", "white"), "black");
    /// ```
    pub fn select<T>(self, dark: T, light: T) -> T {
        match self {
            Self::Light => light,
            Self::Dark => dark,
        }
    }
}

impl Not for Color {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Error correction level

/// The error correction level. It allows the original information be recovered
/// even if parts of the code is damaged.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum EcLevel {
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
/// The smallest version is `Version::Normal(1)` of size 21×21, and the largest
/// is `Version::Normal(40)` of size 177×177.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Version {
    /// A normal QR code version. The parameter should be between 1 and 40.
    Normal(i16),

    /// A Micro QR code version. The parameter should be between 1 and 4.
    Micro(i16),

    /// A rMQR code version. The first parameter represents the height and
    /// should be 7, 9, 11, 13, 15, or 17. The second parameter represents the
    /// width and should be 27, 43, 59, 77, 99, or 139. 27 can only be used with
    /// 11, or 13.
    RectMicro(i16, i16),
}

impl Version {
    /// Gets the number of horizontally-arranged "modules" on each size of the
    /// QR code, i.e. the width of the code.
    ///
    /// Except for rMQR code, the width is the same as the height.
    pub const fn width(self) -> i16 {
        match self {
            Self::Normal(v) => v * 4 + 17,
            Self::Micro(v) => v * 2 + 9,
            Self::RectMicro(_, w) => w,
        }
    }

    /// Gets the number of vertically-arranged "modules" on each size of the QR
    /// code, i.e. the height of the code.
    ///
    /// Except for rMQR code, the height is the same as the width.
    pub const fn height(self) -> i16 {
        if let Self::RectMicro(h, _) = self {
            h
        } else {
            self.width()
        }
    }

    /// Obtains an object from a hard-coded table.
    ///
    /// The table must be a 76×4 array. The outer array represents the content
    /// for each version. The first 40 entry corresponds to QR code versions 1
    /// to 40, the next 4 corresponds to Micro QR code version 1 to 4, and the
    /// last 32 corresponds to rMQR code. The inner array represents the content
    /// in each error correction level, in the order [L, M, Q, H].
    ///
    /// # Errors
    ///
    /// If the entry compares equal to the default value of `T`, this method
    /// returns `Err(QrError::InvalidVersion)`.
    pub fn fetch<T>(self, ec_level: EcLevel, table: &[[T; 4]]) -> QrResult<T>
    where
        T: PartialEq + Default + Copy,
    {
        match self {
            Self::Normal(v @ 1..=40) => {
                return Ok(table[(v - 1).as_usize()][ec_level as usize]);
            }
            Self::Micro(v @ 1..=4) => {
                let obj = table[(v + 39).as_usize()][ec_level as usize];
                if obj != T::default() {
                    return Ok(obj);
                }
            }
            Self::RectMicro(..) => {
                let index = self.rect_micro_index()?;
                let obj = table[index + 44][ec_level as usize];
                if obj != T::default() {
                    return Ok(obj);
                }
            }
            _ => {}
        }
        Err(QrError::InvalidVersion)
    }

    /// The number of bits needed to encode the mode indicator.
    pub fn mode_bits_count(self) -> usize {
        match self {
            Self::Normal(_) => 4,
            Self::Micro(a) => (a - 1).as_usize(),
            Self::RectMicro(..) => 3,
        }
    }

    /// Checks whether is version refers to a normal QR code.
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal(_))
    }

    /// Checks whether is version refers to a Micro QR code.
    pub const fn is_micro(self) -> bool {
        matches!(self, Self::Micro(_))
    }

    /// Checks whether is version refers to a rMQR code.
    pub const fn is_rect_micro(self) -> bool {
        self.rect_micro_index().is_ok()
    }

    /// Gets the index of the version of the rMQR code.
    pub(crate) const fn rect_micro_index(self) -> QrResult<usize> {
        match self {
            Self::RectMicro(7, 43) => Ok(0),
            Self::RectMicro(7, 59) => Ok(1),
            Self::RectMicro(7, 77) => Ok(2),
            Self::RectMicro(7, 99) => Ok(3),
            Self::RectMicro(7, 139) => Ok(4),
            Self::RectMicro(9, 43) => Ok(5),
            Self::RectMicro(9, 59) => Ok(6),
            Self::RectMicro(9, 77) => Ok(7),
            Self::RectMicro(9, 99) => Ok(8),
            Self::RectMicro(9, 139) => Ok(9),
            Self::RectMicro(11, 27) => Ok(10),
            Self::RectMicro(11, 43) => Ok(11),
            Self::RectMicro(11, 59) => Ok(12),
            Self::RectMicro(11, 77) => Ok(13),
            Self::RectMicro(11, 99) => Ok(14),
            Self::RectMicro(11, 139) => Ok(15),
            Self::RectMicro(13, 27) => Ok(16),
            Self::RectMicro(13, 43) => Ok(17),
            Self::RectMicro(13, 59) => Ok(18),
            Self::RectMicro(13, 77) => Ok(19),
            Self::RectMicro(13, 99) => Ok(20),
            Self::RectMicro(13, 139) => Ok(21),
            Self::RectMicro(15, 43) => Ok(22),
            Self::RectMicro(15, 59) => Ok(23),
            Self::RectMicro(15, 77) => Ok(24),
            Self::RectMicro(15, 99) => Ok(25),
            Self::RectMicro(15, 139) => Ok(26),
            Self::RectMicro(17, 43) => Ok(27),
            Self::RectMicro(17, 59) => Ok(28),
            Self::RectMicro(17, 77) => Ok(29),
            Self::RectMicro(17, 99) => Ok(30),
            Self::RectMicro(17, 139) => Ok(31),
            _ => Err(QrError::InvalidVersion),
        }
    }

    /// Gets the index in ascending order of width.
    pub(crate) const fn rect_micro_width_index(self) -> QrResult<usize> {
        match self {
            Self::RectMicro(_, 27) => Ok(0),
            Self::RectMicro(_, 43) => Ok(1),
            Self::RectMicro(_, 59) => Ok(2),
            Self::RectMicro(_, 77) => Ok(3),
            Self::RectMicro(_, 99) => Ok(4),
            Self::RectMicro(_, 139) => Ok(5),
            _ => Err(QrError::InvalidVersion),
        }
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Mode indicator

/// The mode indicator, which specifies the character set of the encoded data.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
    /// ```
    /// use qrcode::types::{Mode, Version};
    ///
    /// assert_eq!(Mode::Numeric.length_bits_count(Version::Normal(1)), 10);
    /// ```
    ///
    /// This method will return `Err(QrError::UnsupportedCharacterSet)` if the
    /// mode is not supported in the given version.
    pub fn length_bits_count(self, version: Version) -> usize {
        match version {
            Version::Micro(a) => {
                let a = a.as_usize();
                match self {
                    Self::Numeric => 2 + a,
                    Self::Alphanumeric | Self::Byte => 1 + a,
                    Self::Kanji => a,
                }
            }
            Version::Normal(1..=9) => match self {
                Self::Numeric => 10,
                Self::Alphanumeric => 9,
                Self::Byte | Self::Kanji => 8,
            },
            Version::Normal(10..=26) => match self {
                Self::Numeric => 12,
                Self::Alphanumeric => 11,
                Self::Byte => 16,
                Self::Kanji => 10,
            },
            Version::Normal(_) => match self {
                Self::Numeric => 14,
                Self::Alphanumeric => 13,
                Self::Byte => 16,
                Self::Kanji => 12,
            },
            Version::RectMicro(..) => {
                let index = version.rect_micro_index().unwrap_or(31);
                match self {
                    Self::Numeric => RMQR_LENGTH_BITS_COUNT[index][0],
                    Self::Alphanumeric => RMQR_LENGTH_BITS_COUNT[index][1],
                    Self::Byte => RMQR_LENGTH_BITS_COUNT[index][2],
                    Self::Kanji => RMQR_LENGTH_BITS_COUNT[index][3],
                }
            }
        }
    }

    /// Computes the number of bits needed to some data of a given raw length.
    ///
    /// ```
    /// use qrcode::types::Mode;
    ///
    /// assert_eq!(Mode::Numeric.data_bits_count(7), 24);
    /// ```
    ///
    /// Note that in Kanji mode, the `raw_data_len` is the number of Kanjis,
    /// i.e. half the total size of bytes.
    pub const fn data_bits_count(self, raw_data_len: usize) -> usize {
        match self {
            Self::Numeric => (raw_data_len * 10 + 2) / 3,
            Self::Alphanumeric => (raw_data_len * 11 + 1) / 2,
            Self::Byte => raw_data_len * 8,
            Self::Kanji => raw_data_len * 13,
        }
    }

    /// Find the lowest common mode which both modes are compatible with.
    ///
    /// ```
    /// use qrcode::types::Mode;
    ///
    /// let a = Mode::Numeric;
    /// let b = Mode::Kanji;
    /// let c = a.max(b);
    /// assert!(a <= c);
    /// assert!(b <= c);
    /// ```
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        match self.partial_cmp(&other) {
            Some(Ordering::Greater) => self,
            Some(_) => other,
            None => Self::Byte,
        }
    }
}

impl PartialOrd for Mode {
    /// Defines a partial ordering between modes. If `a <= b`, then `b` contains
    /// a superset of all characters supported by `a`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (a, b) if a == b => Some(Ordering::Equal),
            (Self::Numeric, Self::Alphanumeric) | (_, Self::Byte) => Some(Ordering::Less),
            (Self::Alphanumeric, Self::Numeric) | (Self::Byte, _) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[cfg(test)]
mod mode_tests {
    use crate::types::Mode::{Alphanumeric, Byte, Kanji, Numeric};

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

/// The number of bits needed to encode the length of the data.
///
/// [Numeric, Alphanumeric, Byte, Kanji]
static RMQR_LENGTH_BITS_COUNT: [[usize; 4]; 32] = [
    [4, 3, 3, 2], // R7x43
    [5, 5, 4, 3], // R7x59
    [6, 5, 5, 4], // R7x77
    [7, 6, 5, 5], // R7x99
    [7, 6, 6, 5], // R7x139
    [5, 5, 4, 3], // R9x43
    [6, 5, 5, 4], // R9x59
    [7, 6, 5, 5], // R9x77
    [7, 6, 6, 5], // R9x99
    [8, 7, 6, 6], // R9x139
    [4, 4, 3, 2], // R11x27
    [6, 5, 5, 4], // R11x43
    [7, 6, 5, 5], // R11x59
    [7, 6, 6, 5], // R11x77
    [8, 7, 6, 6], // R11x99
    [8, 7, 7, 6], // R11x139
    [5, 5, 4, 3], // R13x27
    [6, 6, 5, 5], // R13x43
    [7, 6, 6, 5], // R13x59
    [7, 7, 6, 6], // R13x77
    [8, 7, 7, 6], // R13x99
    [8, 8, 7, 7], // R13x139
    [7, 6, 6, 5], // R15x43
    [7, 7, 6, 5], // R15x59
    [8, 7, 7, 6], // R15x77
    [8, 7, 7, 6], // R15x99
    [9, 8, 7, 7], // R15x139
    [7, 6, 6, 5], // R17x43
    [8, 7, 6, 6], // R17x59
    [8, 7, 7, 6], // R17x77
    [8, 8, 7, 6], // R17x99
    [9, 8, 8, 7], // R17x139
];
