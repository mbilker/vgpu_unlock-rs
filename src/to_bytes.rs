pub trait ToBytes {
    type Bytes: Copy + AsRef<[u8]> + AsMut<[u8]> + IntoIterator<Item = u8> + 'static;

    fn to_ne_bytes(self) -> Self::Bytes;
}

macro_rules! impl_to_bytes {
    ($ty:tt, $len:expr) => {
        impl ToBytes for $ty {
            type Bytes = [u8; $len];

            fn to_ne_bytes(self) -> Self::Bytes {
                $ty::to_ne_bytes(self)
            }
        }
    };
}

impl ToBytes for i8 {
    type Bytes = [u8; 1];

    fn to_ne_bytes(self) -> Self::Bytes {
        [self as u8]
    }
}

impl_to_bytes!(i16, 2);
impl_to_bytes!(i32, 4);
impl_to_bytes!(i64, 8);
#[cfg(target_pointer_width = "32")]
impl_to_bytes!(isize, 4);
#[cfg(target_pointer_width = "64")]
impl_to_bytes!(isize, 8);

impl ToBytes for u8 {
    type Bytes = [u8; 1];

    fn to_ne_bytes(self) -> Self::Bytes {
        [self]
    }
}

impl_to_bytes!(u16, 2);
impl_to_bytes!(u32, 4);
impl_to_bytes!(u64, 8);
#[cfg(target_pointer_width = "32")]
impl_to_bytes!(usize, 4);
#[cfg(target_pointer_width = "64")]
impl_to_bytes!(usize, 8);
