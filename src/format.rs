// SPDX-License-Identifier: MIT

// (`char::decode_utf16` and `char::REPLACEMENT_CHAR` were exposed on the fundamental type
// in Rust 1.52)
use std::char;
use std::fmt::{self, Write};

pub struct CStrFormat<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for CStrFormat<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<'a> fmt::Display for CStrFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = crate::from_c_str(self.0);

        fmt::Debug::fmt(&s, f)
    }
}

pub struct HexFormat<T>(pub T);

impl<T: fmt::LowerHex> fmt::Debug for HexFormat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T: fmt::LowerHex> fmt::Display for HexFormat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

pub struct HexFormatSlice<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for HexFormatSlice<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<'a> fmt::Display for HexFormatSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            f.write_str("[]")
        } else {
            f.write_str("0x")?;

            for v in self.0.iter() {
                write!(f, "{:02x}", v)?;
            }

            Ok(())
        }
    }
}

pub struct WideCharFormat<'a>(pub &'a [u16]);

impl<'a> fmt::Debug for WideCharFormat<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('"')?;

        fmt::Display::fmt(self, f)?;

        f.write_char('"')
    }
}

impl<'a> fmt::Display for WideCharFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in char::decode_utf16(self.0.iter().copied().take_while(|&ch| ch != 0)) {
            f.write_char(item.unwrap_or(char::REPLACEMENT_CHARACTER))?;
        }

        Ok(())
    }
}

pub struct StraightFormat<T>(pub T);

impl<T: fmt::Debug> fmt::Debug for StraightFormat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T: fmt::Display> fmt::Display for StraightFormat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
