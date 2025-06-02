///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrla081.h
use std::fmt;

use super::ctrl2080gpu::{NV2080_GPU_MAX_NAME_STRING_LENGTH, NV_GRID_LICENSE_INFO_MAX_LENGTH};
use crate::format::{CStrFormat, HexFormat, HexFormatSlice, WideCharFormat};
use crate::utils::AlignedU64;

pub const NVA081_VGPU_STRING_BUFFER_SIZE: usize = 32;
pub const NVA081_VGPU_SIGNATURE_SIZE: usize = 128;

pub const NVA081_EXTRA_PARAMETERS_SIZE: usize = 1024;

// pub const NVA081_MAX_VGPU_PER_PGPU: usize = 32;

/// See `NVA081_CTRL_VGPU_CONFIG_INFO`
// Set `align(8)` for `NVA081_CTRL_VGPU_CONFIG_GET_VGPU_TYPE_INFO_PARAMS`
#[repr(C, align(8))]
pub struct NvA081CtrlVgpuInfo {
    pub vgpu_type: u32,
    pub vgpu_name: [u8; NVA081_VGPU_STRING_BUFFER_SIZE],
    pub vgpu_class: [u8; NVA081_VGPU_STRING_BUFFER_SIZE],
    pub vgpu_signature: [u8; NVA081_VGPU_SIGNATURE_SIZE],
    pub license: [u8; NV_GRID_LICENSE_INFO_MAX_LENGTH],
    pub max_instance: u32,
    pub num_heads: u32,
    pub max_resolution_x: u32,
    pub max_resolution_y: u32,
    pub max_pixels: u32,
    pub frl_config: u32,
    pub cuda_enabled: u32,
    pub ecc_supported: u32,
    pub gpu_instance_size: u32,
    pub multi_vgpu_supported: u32,
    pub vdev_id: AlignedU64,
    pub pdev_id: AlignedU64,
    pub profile_size: AlignedU64,
    pub fb_length: AlignedU64,
    pub gsp_heap_size: AlignedU64,
    pub fb_reservation: AlignedU64,
    pub mappable_video_size: AlignedU64,
    pub encoder_capacity: u32,
    pub bar1_length: AlignedU64,
    pub frl_enable: u32,
    pub adapter_name: [u8; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub adapter_name_unicode: [u16; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub short_gpu_name_string: [u8; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub licensed_product_name: [u8; NV_GRID_LICENSE_INFO_MAX_LENGTH],
    pub vgpu_extra_params: [u32; NVA081_EXTRA_PARAMETERS_SIZE],
    pub ftrace_enable: u32,
    pub gpu_direct_supported: u32,
    pub nvlink_p2p_supported: u32,
    pub multi_vgpu_exclusive: u32,
    pub exclusive_type: u32,
    pub exclusive_size: u32,
    pub gpu_instance_profile_id: u32,
    // R550 adds additional fields, leave them out for now for backwards compat with 16.x
    // https://github.com/NVIDIA/open-gpu-kernel-modules/blob/550/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L126-L128
    // R570 rename these fields
    // https://github.com/NVIDIA/open-gpu-kernel-modules/blob/570/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L126-L128
    //
    // pub placement_size: u32,
    // pub homogeneousPlacementCount: u32, // pub placement_count: u32,
    // pub homogeneousPlacementIds: [u32; NVA081_MAX_VGPU_PER_PGPU], // pub placement_ids: [u32; NVA081_MAX_VGPU_PER_PGPU],
    //
    // R570 adds additional fields, leave them out for now for backwards compat with 16.x and 17.x
    // https://github.com/NVIDIA/open-gpu-kernel-modules/blob/570/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L129-L130
    //
    // pub heterogeneousPlacementCount: u32,
    // pub heterogeneousPlacementIds: [u32; NVA081_MAX_VGPU_PER_PGPU],
}

pub const NVA081_CTRL_CMD_VGPU_CONFIG_GET_VGPU_TYPE_INFO: u32 = 0xa0810103;

/// This RM control command is used starting in vGPU version 15.0 (525.60.12).
///
/// See `NVA081_CTRL_VGPU_CONFIG_GET_VGPU_TYPE_INFO_PARAMS`
#[repr(C)]
pub struct NvA081CtrlVgpuConfigGetVgpuTypeInfoParams {
    pub vgpu_type: u32,
    pub vgpu_type_info: NvA081CtrlVgpuInfo,
}

pub const NVA081_CTRL_CMD_VGPU_CONFIG_GET_MIGRATION_CAP: u32 = 0xa0810112;

/// See `NVA081_CTRL_CMD_VGPU_CONFIG_GET_MIGRATION_CAP_PARAMS`
#[repr(C)]
pub struct NvA081CtrlCmdVgpuConfigGetMigrationCapParams {
    pub migration_cap: u8,
}

impl fmt::Debug for NvA081CtrlVgpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vgpu_signature = if self.vgpu_signature[..].iter().any(|&x| x != 0) {
            &self.vgpu_signature[..]
        } else {
            &[]
        };
        let vgpu_extra_params = if self.vgpu_extra_params[..].iter().any(|&x| x != 0) {
            &self.vgpu_extra_params[..]
        } else {
            &[]
        };

        f.debug_struct("NvA081CtrlVgpuInfo")
            .field("vgpu_type", &self.vgpu_type)
            .field("vgpu_name", &CStrFormat(&self.vgpu_name))
            .field("vgpu_class", &CStrFormat(&self.vgpu_class))
            .field("vgpu_signature", &HexFormatSlice(vgpu_signature))
            .field("license", &CStrFormat(&self.license))
            .field("max_instance", &self.max_instance)
            .field("num_heads", &self.num_heads)
            .field("max_resolution_x", &self.max_resolution_x)
            .field("max_resolution_y", &self.max_resolution_y)
            .field("max_pixels", &self.max_pixels)
            .field("frl_config", &self.frl_config)
            .field("cuda_enabled", &self.cuda_enabled)
            .field("ecc_supported", &self.ecc_supported)
            .field("gpu_instance_size", &self.gpu_instance_size)
            .field("multi_vgpu_supported", &self.multi_vgpu_supported)
            .field("vdev_id", &HexFormat(self.vdev_id))
            .field("pdev_id", &HexFormat(self.pdev_id))
            .field("profile_size", &HexFormat(self.profile_size))
            .field("fb_length", &HexFormat(self.fb_length))
            .field("gsp_heap_size", &HexFormat(self.gsp_heap_size))
            .field("fb_reservation", &HexFormat(self.fb_reservation))
            .field("mappable_video_size", &HexFormat(self.mappable_video_size))
            .field("encoder_capacity", &HexFormat(self.encoder_capacity))
            .field("bar1_length", &HexFormat(self.bar1_length))
            .field("frl_enable", &self.frl_enable)
            .field("adapter_name", &CStrFormat(&self.adapter_name))
            .field(
                "adapter_name_unicode",
                &WideCharFormat(&self.adapter_name_unicode),
            )
            .field(
                "short_gpu_name_string",
                &CStrFormat(&self.short_gpu_name_string),
            )
            .field(
                "licensed_product_name",
                &CStrFormat(&self.licensed_product_name),
            )
            .field("vgpu_extra_params", &HexFormatSlice(vgpu_extra_params))
            .field("ftrace_enable", &self.ftrace_enable)
            .field("gpu_direct_supported", &self.gpu_direct_supported)
            .field("nvlink_p2p_supported", &self.nvlink_p2p_supported)
            .field("multi_vgpu_exclusive", &self.multi_vgpu_exclusive)
            .field("exclusive_type", &self.exclusive_type)
            .field("exclusive_size", &self.exclusive_size)
            .field("gpu_instance_profile_id", &self.gpu_instance_profile_id)
            .finish()
    }
}

impl fmt::Debug for NvA081CtrlVgpuConfigGetVgpuTypeInfoParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NvA081CtrlVgpuConfigGetVgpuTypeInfoParams")
            .field("vgpu_type", &self.vgpu_type)
            .field("vgpu_type_info", &self.vgpu_type_info)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::{NvA081CtrlVgpuConfigGetVgpuTypeInfoParams, NvA081CtrlVgpuInfo};

    #[test]
    fn verify_sizes() {
        assert_eq!(mem::size_of::<NvA081CtrlVgpuInfo>(), 0x1358);
        assert_eq!(
            mem::size_of::<NvA081CtrlVgpuConfigGetVgpuTypeInfoParams>(),
            0x1360
        );
    }
}
