use std::fmt;

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct AlignedU64(pub u64);

impl fmt::Debug for AlignedU64 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for AlignedU64 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}
