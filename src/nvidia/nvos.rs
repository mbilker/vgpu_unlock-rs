///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/nvos.h
use std::os::raw::{c_ulong, c_void};

use super::ioctl::NV_IOCTL_MAGIC;
use super::nvtypes::NvHandle;
use crate::ioctl::_IOWR;

/// Value of the "request" argument used by `nvidia-vgpud` and `nvidia-vgpu-mgr` when calling
/// ioctl to read the PCI device ID, type, and many other things from the driver.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/nvidia/arch/nvalloc/unix/include/nv_escape.h
/// and [`nvidia_ioctl`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/nvidia/nv.c)
/// and [`__NV_IOWR`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/common/inc/nv.h)
/// showing that `_IOWR` is used to derive the I/O control request codes.
pub const NV_ESC_RM_CONTROL: c_ulong = _IOWR::<Nvos54Parameters>(NV_IOCTL_MAGIC, 0x2a);

/// When issuing ioctl with `NV_ESC_RM_CONTROL` then the `argp` argument is a pointer to a
/// `NVOS54_PARAMETERS` structure like this.
///
/// See [`NVOS54_PARAMETERS`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/nvos.h)
//#[derive(Debug)]
#[repr(C)]
pub struct Nvos54Parameters {
    /// Initialized prior to call.
    pub h_client: NvHandle,
    /// Initialized prior to call.
    pub h_object: NvHandle,
    /// Operation type, see comment below.
    pub cmd: u32,
    /// Set of flags for call.
    pub flags: u32,
    /// Pointer initialized prior to call.
    /// Pointee initialized to 0 prior to call.
    /// Pointee is written by ioctl call.
    pub params: *mut c_void,
    /// Size in bytes of the object referenced in `params`.
    pub params_size: u32,
    /// Written by ioctl call. See comment below.
    pub status: u32,
}
