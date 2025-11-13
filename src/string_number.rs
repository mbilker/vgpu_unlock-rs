use std::borrow::Cow;
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::de::{Deserializer, Error};
use serde::Deserialize;

#[repr(transparent)]
pub struct U32(pub u32);

impl<'de> Deserialize<'de> for U32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match NumberString::deserialize(deserializer)? {
            NumberString::Number(n) => Ok(Self(n)),
            NumberString::String(s) => {
                let s = s.trim();

                // Try to maintain compatibility with older Rust versions
                let (v, radix) = match (s.get(0..2), s.get(2..)) {
                    (Some(prefix), Some(suffix)) if prefix.eq_ignore_ascii_case("0b") => {
                        (suffix, 2)
                    }
                    (Some(prefix), Some(suffix)) if prefix.eq_ignore_ascii_case("0x") => {
                        (suffix, 16)
                    }
                    (_, _) => (s, 10),
                };

                match u32::from_str_radix(v, radix) {
                    Ok(n) => Ok(Self(n)),
                    Err(e) => Err(D::Error::custom(format!(
                        "Failed to parse string as base-{} integer: {}",
                        radix, e
                    ))),
                }
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberString<'data> {
    Number(u32),
    #[serde(borrow)]
    String(Cow<'data, str>),
}

impl fmt::Display for U32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for U32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Eq for U32 {}

impl Hash for U32 {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        Hash::hash(&self.0, state)
    }
}

impl PartialEq<u32> for U32 {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        PartialEq::eq(&self.0, other)
    }

    #[inline]
    fn ne(&self, other: &u32) -> bool {
        PartialEq::ne(&self.0, other)
    }
}

impl PartialEq<U32> for U32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self.0, &other.0)
    }
}
