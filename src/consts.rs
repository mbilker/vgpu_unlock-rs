use std::ffi::c_ulong;

use crate::{ioctl::_IOCWR, structs::Nvos54Parameters};

pub const DEFAULT_CONFIG_PATH: &str = "/etc/vgpu_unlock/config.toml";
pub const DEFAULT_PROFILE_OVERRIDE_CONFIG_PATH: &str = "/etc/vgpu_unlock/profile_override.toml";

/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/kernel-open/common/inc/nv-ioctl-numbers.h
pub const NV_IOCTL_MAGIC: c_ulong = b'F' as _;

/// Value of the "request" argument used by `nvidia-vgpud` and `nvidia-vgpu-mgr` when calling
/// ioctl to read the PCI device ID, type, and many other things from the driver.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/nvidia/arch/nvalloc/unix/include/nv_escape.h
/// and [`nvidia_ioctl`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/nvidia/nv.c)
/// and [`__NV_IOWR`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/common/inc/nv.h)
/// showing that `_IOCWR` is used to derive the I/O control request codes.
pub const NV_ESC_RM_CONTROL: c_ulong = _IOCWR::<Nvos54Parameters>(NV_IOCTL_MAGIC, 0x2a);

/// `result` is a pointer to [`Nv0000CtrlVgpuGetStartDataParams`].
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrl0000/ctrl0000vgpu.h#L65
pub const NV0000_CTRL_CMD_VGPU_GET_START_DATA: u32 = 0xc01;

/// `result` is a pointer to `uint32_t`.
pub const NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE: u32 = 0x800289;

/// `result` is a pointer to `uint32_t[4]`, the second element (index 1) is the device ID, the
/// forth element (index 3) is the subsystem ID.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/ctrl/ctrl2080/ctrl2080bus.h
pub const NV2080_CTRL_CMD_BUS_GET_PCI_INFO: u32 = 0x20801801;

/// `result` is a pointer to `VgpuConfig`.
pub const OP_READ_VGPU_CFG: u32 = 0xa0820102;

/// `result` is a pointer to [`NVA081_CTRL_VGPU_CONFIG_GET_VGPU_TYPE_INFO_PARAMS`].
///
/// This RM control command is used starting in vGPU version 15.0 (525.60.12).
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L298
pub const NVA081_CTRL_CMD_VGPU_CONFIG_GET_VGPU_TYPE_INFO: u32 = 0xA0810103;

/// `result` is a pointer to `bool`.
pub const OP_READ_VGPU_MIGRATION_CAP: u32 = 0xa0810112;

/// `nvidia-vgpu-mgr` expects this value for a vGPU capable GPU.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/ctrl/ctrl0080/ctrl0080gpu.h
pub const NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST: u32 = 3;

/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/5f40a5aee5ef9c92085836bf5b5a9056174f07f1/src/common/sdk/nvidia/inc/ctrl/ctrl2080/ctrl2080gpu.h#L1772
pub const NV2080_CTRL_CMD_GPU_GET_INFOROM_OBJECT_VERSION: u32 = 0x2080014B;

/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/90eb10774f1c53d2364eacf9fa8f0c7a92b1b824/src/common/sdk/nvidia/inc/ctrl/ctrl9096.h#L226
pub const NV9096_CTRL_CMD_GET_ZBC_CLEAR_TABLE: u32 = 0x90960103;

/// When ioctl returns success (retval >= 0) but sets the status value of the arg structure to 3
/// then `nvidia-vgpud` will sleep for a bit (first 0.1s then 1s then 10s) then issue the same
/// ioctl call again until the status differs from 3. It will attempt this for up to 24h before
/// giving up.
///
/// See https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/kernel-open/common/inc/nvstatuscodes.h
pub const NV_OK: u32 = 0x0;
pub const NV_ERR_BUSY_RETRY: u32 = 0x3;
pub const NV_ERR_NOT_SUPPORTED: u32 = 0x56;
pub const NV_ERR_OBJECT_NOT_FOUND: u32 = 0x57;

pub const NVA081_VGPU_STRING_BUFFER_SIZE: usize = 32;
pub const NVA081_VGPU_SIGNATURE_SIZE: usize = 128;
pub const NVA081_EXTRA_PARAMETERS_SIZE: usize = 1024;
pub const NV_GRID_LICENSE_INFO_MAX_LENGTH: usize = 128;
pub const NV2080_GPU_MAX_NAME_STRING_LENGTH: usize = 64;
