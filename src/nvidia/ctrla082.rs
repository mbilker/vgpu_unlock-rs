use std::fmt;

use crate::format::{CStrFormat, HexFormat, HexFormatSlice, WideCharFormat};

/// Inferred based on `NVA082_CTRL_CMD_HOST_VGPU_DEVICE_GET_VGPU_TYPE_INFO_PARAMS`
pub const NVA082_CTRL_CMD_HOST_VGPU_DEVICE_GET_VGPU_TYPE_INFO: u32 = 0xa0820102;

/// Pulled from a comment in [`NVA081_CTRL_VGPU_INFO`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L82)
#[repr(C)]
pub struct NvA082CtrlCmdHostVgpuDeviceGetVgpuTypeInfoParams {
    pub vgpu_type: u32,
    pub vgpu_name: [u8; 32],
    pub vgpu_class: [u8; 32],
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
    // This field might not exist anymore and instead the space became padding as
    // `NVA081_CTRL_VGPU_INFO` forces the alignment of `vdevId` to `8`.
    pub mig_instance_size: u32,
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
    unknown_end: [u8; 8],
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
            .field("mig_instance_size", &self.mig_instance_size)
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
            0x738
        );
    }
}
