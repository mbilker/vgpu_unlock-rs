// SPDX-License-Identifier: MIT

use std::fmt;

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
