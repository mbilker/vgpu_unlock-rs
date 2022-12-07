// SPDX-FileContributor: musl
// SPDX-License-Identifier: MIT
//
// Derived from musl 1.2.3 `generic/bits/ioctl.h`. musl 1.2.3 is MIT licensed as well so deriving
// constant and function definitions here from musl 1.2.3 should be alright.

use std::mem;

use libc::c_ulong;

const _IOC_WRITE: c_ulong = 1;
const _IOC_READ: c_ulong = 2;

#[allow(non_snake_case)]
#[inline]
pub const fn _IOC(a: c_ulong, b: c_ulong, c: c_ulong, d: c_ulong) -> c_ulong {
    a << 30 | b << 8 | c | d << 16
}

#[allow(non_snake_case)]
#[inline]
pub const fn _IOWR<T>(b: c_ulong, c: c_ulong) -> c_ulong {
    _IOC(_IOC_READ | _IOC_WRITE, b, c, mem::size_of::<T>() as c_ulong)
}
