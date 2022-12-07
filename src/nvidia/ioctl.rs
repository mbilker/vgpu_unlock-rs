use std::os::raw::c_ulong;

/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/kernel-open/common/inc/nv-ioctl-numbers.h
pub const NV_IOCTL_MAGIC: c_ulong = b'F' as _;
