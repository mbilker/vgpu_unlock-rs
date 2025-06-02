///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrl0080/ctrl0080gpu.h

pub const NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE: u32 = 0x800289;

/// `nvidia-vgpu-mgr` expects this value for a vGPU capable GPU.
pub const NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST: u32 = 0x00000003;

/// See `NV0080_CTRL_GPU_GET_VIRTUALIZATION_MODE_PARAMS`
#[repr(C)]
pub struct Nv0080CtrlGpuGetVirtualizationModeParams {
    pub virtualization_mode: u32,
    // R570 adds additional fields, leave them out for now for backwards compat with 16.x and 17.x
    // https://github.com/NVIDIA/open-gpu-kernel-modules/blob/570/src/common/sdk/nvidia/inc/ctrl/ctrl0080/ctrl0080gpu.h#L313
    //
    // pub isGridBuild: bool,
}
