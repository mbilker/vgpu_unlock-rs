///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/307159f2623d3bf45feb9177bd2da52ffbc5ddf9/src/common/sdk/nvidia/inc/ctrl/ctrl0080/ctrl0080gpu.h

pub const NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE: u32 = 0x800289;

/// `nvidia-vgpu-mgr` expects this value for a vGPU capable GPU.
pub const NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST: u32 = 0x00000003;

/// See `NV0080_CTRL_GPU_GET_VIRTUALIZATION_MODE_PARAMS`
#[repr(C)]
pub struct Nv0080CtrlGpuGetVirtualizationModeParams {
    pub virtualization_mode: u32,
    pub is_grid_build: bool,
}
