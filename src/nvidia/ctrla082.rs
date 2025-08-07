use std::fmt;

use crate::format::{CStrFormat, HexFormat, HexFormatSlice, WideCharFormat};

/// Inferred based on `NVA082_CTRL_CMD_HOST_VGPU_DEVICE_GET_VGPU_TYPE_INFO_PARAMS`
pub const NVA082_CTRL_CMD_HOST_VGPU_DEVICE_GET_VGPU_TYPE_INFO: u32 = 0xa0820102;

/// Pulled from a comment in [`NVA081_CTRL_VGPU_INFO`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/307159f2623d3bf45feb9177bd2da52ffbc5ddf9/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L89)
#[repr(C)]
pub struct NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams {
    pub vgpu_type: u32,
    pub vgpu_name: [u8; 64],
    pub vgpu_class: [u8; 64],
    pub vgpu_signature: [u8; 128],
    pub license: [u8; 128],
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
    pub vdev_id: u64,
    pub pdev_id: u64,
    pub fb_length: u64,
    pub mappable_video_size: u64,
    pub fb_reservation: u64,
    pub encoder_capacity: u32,
    pub bar1_length: u64,
    pub frl_enable: u32,
    pub adapter_name: [u8; 64],
    pub adapter_name_unicode: [u16; 64],
    pub short_gpu_name_string: [u8; 64],
    pub licensed_product_name: [u8; 128],
    pub vgpu_extra_params: [u8; 1024],
    pub ftrace_enable: u32,
    pub gpu_direct_supported: u32,
    pub nvlink_p2p_supported: u32,
    pub max_instance_per_gi: u32,
    pub multi_vgpu_exclusive: u32,
    pub exclusive_type: u32,
    pub exclusive_size: u32,
    pub gpu_instance_profile_id: u32,
    pub placement_size: u32,
    pub homogeneous_placement_count: u32,
    pub homogeneous_placement_ids: [u32; 48],
    pub heterogeneous_placemen_count: u32,
    pub heterogeneous_placement_ids: [u32; 48],
}

impl fmt::Debug for NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams {
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

        f.debug_struct("NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams")
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
            .field("fb_length", &HexFormat(self.fb_length))
            .field("mappable_video_size", &HexFormat(self.mappable_video_size))
            .field("fb_reservation", &HexFormat(self.fb_reservation))
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
            .finish()
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams;

    #[test]
    fn verify_sizes() {
        assert_eq!(
            mem::size_of::<NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams>(),
            0x918
        );
    }
}
