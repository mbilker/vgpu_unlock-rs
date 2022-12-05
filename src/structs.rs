use std::{collections::HashMap, fmt, mem::size_of};

use libc::c_void;
use serde::Deserialize;

use crate::{
    consts::{
        NV2080_GPU_MAX_NAME_STRING_LENGTH, NVA081_EXTRA_PARAMETERS_SIZE,
        NVA081_VGPU_SIGNATURE_SIZE, NVA081_VGPU_STRING_BUFFER_SIZE,
        NV_GRID_LICENSE_INFO_MAX_LENGTH,
    },
    format::{CStrFormat, HexFormat, HexFormatSlice, WideCharFormat},
    human_number,
};

/// When issuing ioctl with `NV_ESC_RM_CONTROL` then the `argp` argument is a pointer to a
/// `NVOS54_PARAMETERS` structure like this.
///
/// See [`NVOS54_PARAMETERS`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/nvos.h)
//#[derive(Debug)]
#[repr(C)]
pub struct Nvos54Parameters {
    /// Initialized prior to call.
    pub h_client: u32,
    /// Initialized prior to call.
    pub h_object: u32,
    /// Operation type, see comment below.
    pub cmd: u32,
    /// Pointer initialized prior to call.
    /// Pointee initialized to 0 prior to call.
    /// Pointee is written by ioctl call.
    pub params: *mut c_void,
    /// Size in bytes of the object referenced in `params`.
    pub params_size: u32,
    /// Written by ioctl call. See comment below.
    pub status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct Uuid(u32, u16, u16, [u8; 8]);

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0,
            self.1,
            self.2,
            self.3[0],
            self.3[1],
            self.3[2],
            self.3[3],
            self.3[4],
            self.3[5],
            self.3[6],
            self.3[7]
        )
    }
}

/// See [NV0000_CTRL_VGPU_GET_START_DATA_PARAMS](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrl0000/ctrl0000vgpu.h#L69)
#[repr(C)]
pub(crate) struct Nv0000CtrlVgpuGetStartDataParams {
    pub uuid: Uuid,
    pub config_params: [u8; 1024],
    pub qemu_pid: u32,
    pub gpu_pci_id: u32,
    pub vgpu_id: u32,
    pub gpu_pci_bdf: u32,
}

#[repr(C)]
pub(crate) struct VgpuConfig {
    pub vgpu_type: u32,
    pub vgpu_name: [u8; 32],
    pub vgpu_class: [u8; 32],
    pub vgpu_signature: [u8; 128],
    pub features: [u8; 128],
    pub max_instances: u32,
    pub num_heads: u32,
    pub max_resolution_x: u32,
    pub max_resolution_y: u32,
    pub max_pixels: u32,
    pub frl_config: u32,
    pub cuda_enabled: u32,
    pub ecc_supported: u32,
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
}

/// See [NVA081_CTRL_VGPU_INFO](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L81)
#[repr(C)]
pub struct Nva081CtrlVgpuInfo {
    pub vgpu_type: u32,
    pub vgpu_name: [u8; NVA081_VGPU_STRING_BUFFER_SIZE],
    pub vgpu_class: [u8; NVA081_VGPU_STRING_BUFFER_SIZE],
    pub vgpu_signature: [u8; NVA081_VGPU_SIGNATURE_SIZE],
    pub features: [u8; NV_GRID_LICENSE_INFO_MAX_LENGTH],
    pub max_instances: u32,
    pub num_heads: u32,
    pub max_resolution_x: u32,
    pub max_resolution_y: u32,
    pub max_pixels: u32,
    pub frl_config: u32,
    pub cuda_enabled: u32,
    pub ecc_supported: u32,
    pub mig_instance_size: u32,
    pub multi_vgpu_supported: u32,
    pub vdev_id: u64,
    pub pdev_id: u64,
    pub profile_size: u64,
    pub fb_length: u64,
    pub gsp_heap_size: u64,
    pub fb_reservation: u64,
    pub mappable_video_size: u64,
    pub encoder_capacity: u32,
    pub bar1_length: u64,
    pub frl_enable: u32,
    pub adapter_name: [u8; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub adapter_name_unicode: [u16; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub short_gpu_name_string: [u8; NV2080_GPU_MAX_NAME_STRING_LENGTH],
    pub licensed_product_name: [u8; NV_GRID_LICENSE_INFO_MAX_LENGTH],
    /// This is declared as an array of NvU32, but is being used as a string buffer
    /// in [nvidia code](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/nvidia/arch/nvalloc/unix/src/os-hypervisor.c#L550) so do that too for ease of use
    pub vgpu_extra_params: [u8; NVA081_EXTRA_PARAMETERS_SIZE * size_of::<u32>()],
    pub ftrace_enable: u32,
    pub gpu_direct_supported: u32,
    pub nvlink_p2p_supported: u32,
    pub multi_vgpu_exclusive: u32,
    pub exclusive_type: u32,
    pub exclusive_size: u32,
    pub gpu_instance_profile_id: u32,
}

/// See [NVA081_CTRL_VGPU_CONFIG_GET_VGPU_TYPE_INFO_PARAMS](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrla081.h#L306)
#[repr(C)]
#[derive(Debug)]
pub struct Nva081CtrlVgpuConfigGetVgpuTypeInfoParams {
    pub vgpu_type: u32,
    pub vgpu_type_info: Nva081CtrlVgpuInfo,
}

pub trait VgpuConfigLike {
    fn vgpu_type(&mut self) -> &mut u32;
    fn vgpu_name(&mut self) -> &mut [u8; 32];
    fn vgpu_class(&mut self) -> &mut [u8; 32];
    fn vgpu_signature(&mut self) -> &mut [u8; 128];
    fn features(&mut self) -> &mut [u8; 128];
    fn max_instances(&mut self) -> &mut u32;
    fn num_heads(&mut self) -> &mut u32;
    fn max_resolution_x(&mut self) -> &mut u32;
    fn max_resolution_y(&mut self) -> &mut u32;
    fn max_pixels(&mut self) -> &mut u32;
    fn frl_config(&mut self) -> &mut u32;
    fn cuda_enabled(&mut self) -> &mut u32;
    fn ecc_supported(&mut self) -> &mut u32;
    fn mig_instance_size(&mut self) -> &mut u32;
    fn multi_vgpu_supported(&mut self) -> &mut u32;
    fn vdev_id(&mut self) -> &mut u64;
    fn pdev_id(&mut self) -> &mut u64;
    fn profile_size(&mut self) -> Option<&mut u64>;
    fn fb_length(&mut self) -> &mut u64;
    fn mappable_video_size(&mut self) -> &mut u64;
    fn fb_reservation(&mut self) -> &mut u64;
    fn encoder_capacity(&mut self) -> &mut u32;
    fn bar1_length(&mut self) -> &mut u64;
    fn frl_enable(&mut self) -> &mut u32;
    fn adapter_name(&mut self) -> &mut [u8; 64];
    fn adapter_name_unicode(&mut self) -> &mut [u16; 64];
    fn short_gpu_name_string(&mut self) -> &mut [u8; 64];
    fn licensed_product_name(&mut self) -> &mut [u8; 128];
    /// Return a slice instead of reference to array because VgpuConfig and Nva081CtrlVgpuInfo
    /// have different sizes for vgpu_extra_params
    fn vgpu_extra_params(&mut self) -> &mut [u8];
}

macro_rules! impl_trait_fn {
    ($name:ident, $t:ty) => {
        fn $name(&mut self) -> &mut $t {
            &mut self.$name
        }
    };
}

impl VgpuConfigLike for VgpuConfig {
    impl_trait_fn!(vgpu_type, u32);
    impl_trait_fn!(vgpu_name, [u8; 32]);
    impl_trait_fn!(vgpu_class, [u8; 32]);
    impl_trait_fn!(vgpu_signature, [u8; 128]);
    impl_trait_fn!(features, [u8; 128]);
    impl_trait_fn!(max_instances, u32);
    impl_trait_fn!(num_heads, u32);
    impl_trait_fn!(max_resolution_x, u32);
    impl_trait_fn!(max_resolution_y, u32);
    impl_trait_fn!(max_pixels, u32);
    impl_trait_fn!(frl_config, u32);
    impl_trait_fn!(cuda_enabled, u32);
    impl_trait_fn!(ecc_supported, u32);
    impl_trait_fn!(mig_instance_size, u32);
    impl_trait_fn!(multi_vgpu_supported, u32);
    impl_trait_fn!(vdev_id, u64);
    impl_trait_fn!(pdev_id, u64);

    fn profile_size(&mut self) -> Option<&mut u64> {
        None
    }

    impl_trait_fn!(fb_length, u64);
    impl_trait_fn!(mappable_video_size, u64);
    impl_trait_fn!(fb_reservation, u64);
    impl_trait_fn!(encoder_capacity, u32);
    impl_trait_fn!(bar1_length, u64);
    impl_trait_fn!(frl_enable, u32);
    impl_trait_fn!(adapter_name, [u8; 64]);
    impl_trait_fn!(adapter_name_unicode, [u16; 64]);
    impl_trait_fn!(short_gpu_name_string, [u8; 64]);
    impl_trait_fn!(licensed_product_name, [u8; 128]);
    impl_trait_fn!(vgpu_extra_params, [u8]);
}

impl VgpuConfigLike for Nva081CtrlVgpuInfo {
    impl_trait_fn!(vgpu_type, u32);
    impl_trait_fn!(vgpu_name, [u8; 32]);
    impl_trait_fn!(vgpu_class, [u8; 32]);
    impl_trait_fn!(vgpu_signature, [u8; 128]);
    impl_trait_fn!(features, [u8; 128]);
    impl_trait_fn!(max_instances, u32);
    impl_trait_fn!(num_heads, u32);
    impl_trait_fn!(max_resolution_x, u32);
    impl_trait_fn!(max_resolution_y, u32);
    impl_trait_fn!(max_pixels, u32);
    impl_trait_fn!(frl_config, u32);
    impl_trait_fn!(cuda_enabled, u32);
    impl_trait_fn!(ecc_supported, u32);
    impl_trait_fn!(mig_instance_size, u32);
    impl_trait_fn!(multi_vgpu_supported, u32);
    impl_trait_fn!(vdev_id, u64);
    impl_trait_fn!(pdev_id, u64);

    fn profile_size(&mut self) -> Option<&mut u64> {
        Some(&mut self.profile_size)
    }

    impl_trait_fn!(fb_length, u64);
    impl_trait_fn!(mappable_video_size, u64);
    impl_trait_fn!(fb_reservation, u64);
    impl_trait_fn!(encoder_capacity, u32);
    impl_trait_fn!(bar1_length, u64);
    impl_trait_fn!(frl_enable, u32);
    impl_trait_fn!(adapter_name, [u8; 64]);
    impl_trait_fn!(adapter_name_unicode, [u16; 64]);
    impl_trait_fn!(short_gpu_name_string, [u8; 64]);
    impl_trait_fn!(licensed_product_name, [u8; 128]);
    impl_trait_fn!(vgpu_extra_params, [u8]);
}

#[derive(Deserialize)]
pub struct ProfileOverridesConfig<'a> {
    #[serde(borrow, default)]
    pub profile: HashMap<&'a str, VgpuProfileOverride<'a>>,
    #[serde(borrow, default)]
    pub mdev: HashMap<&'a str, VgpuProfileOverride<'a>>,
}

#[derive(Deserialize)]
pub struct VgpuProfileOverride<'a> {
    pub gpu_type: Option<u32>,
    pub card_name: Option<&'a str>,
    pub vgpu_type: Option<&'a str>,
    pub features: Option<&'a str>,
    pub max_instances: Option<u32>,
    pub num_displays: Option<u32>,
    pub display_width: Option<u32>,
    pub display_height: Option<u32>,
    pub max_pixels: Option<u32>,
    pub frl_config: Option<u32>,
    pub cuda_enabled: Option<u32>,
    pub ecc_supported: Option<u32>,
    pub mig_instance_size: Option<u32>,
    pub multi_vgpu_supported: Option<u32>,
    pub pci_id: Option<u64>,
    pub pci_device_id: Option<u64>,
    #[serde(default, with = "human_number")]
    pub framebuffer: Option<u64>,
    #[serde(default, with = "human_number")]
    pub mappable_video_size: Option<u64>,
    #[serde(default, with = "human_number")]
    pub framebuffer_reservation: Option<u64>,
    pub encoder_capacity: Option<u32>,
    pub bar1_length: Option<u64>,
    pub frl_enabled: Option<u32>,
    pub adapter_name: Option<&'a str>,
    pub short_gpu_name: Option<&'a str>,
    pub license_type: Option<&'a str>,
}

impl fmt::Debug for Nv0000CtrlVgpuGetStartDataParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VgpuStart")
            .field("uuid", &format_args!("{{{}}}", self.uuid))
            .field("config_params", &CStrFormat(&self.config_params))
            .field("qemu_pid", &self.qemu_pid)
            .field("gpu_pci_id", &self.gpu_pci_id)
            .field("vgpu_id", &self.vgpu_id)
            .field("gpu_pci_bdf", &self.gpu_pci_bdf)
            .finish()
    }
}

impl fmt::Debug for VgpuConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vgpu_signature = self.vgpu_signature[..]
            .split(|&x| x == 0)
            .next()
            .unwrap_or(&[]);
        let vgpu_extra_params = self.vgpu_extra_params[..]
            .split(|&x| x == 0)
            .next()
            .unwrap_or(&[]);

        f.debug_struct("VgpuConfig")
            .field("vgpu_type", &self.vgpu_type)
            .field("vgpu_name", &CStrFormat(&self.vgpu_name))
            .field("vgpu_class", &CStrFormat(&self.vgpu_class))
            .field("vgpu_signature", &HexFormatSlice(vgpu_signature))
            .field("features", &CStrFormat(&self.features))
            .field("max_instances", &self.max_instances)
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

impl fmt::Debug for Nva081CtrlVgpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vgpu_signature = self.vgpu_signature[..]
            .split(|&x| x == 0)
            .next()
            .unwrap_or(&[]);
        let vgpu_extra_params = self.vgpu_extra_params[..]
            .split(|&x| x == 0)
            .next()
            .unwrap_or(&[]);

        f.debug_struct("Nva081CtrlVgpuInfo")
            .field("vgpu_type", &self.vgpu_type)
            .field("vgpu_name", &CStrFormat(&self.vgpu_name))
            .field("vgpu_class", &CStrFormat(&self.vgpu_class))
            .field("vgpu_signature", &HexFormatSlice(vgpu_signature))
            .field("features", &CStrFormat(&self.features))
            .field("max_instances", &self.max_instances)
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
            .field("vgpu_extra_params", &CStrFormat(vgpu_extra_params))
            .finish()
    }
}

/// See [NV2080_CTRL_BUS_GET_PCI_INFO_PARAMS](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrl2080/ctrl2080bus.h#L67)
#[repr(C)]
pub struct Nv2080CtrlBusGetPciInfoParams {
    pub pci_device_id: u32,
    pub pci_sub_system_id: u32,
    pub pci_revision_id: u32,
    pub pci_ext_device_id: u32,
} 
