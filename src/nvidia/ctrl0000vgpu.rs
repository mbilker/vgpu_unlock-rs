///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/758b4ee8189c5198504cb1c3c5bc29027a9118a3/src/common/sdk/nvidia/inc/ctrl/ctrl0000/ctrl0000vgpu.h
use std::fmt;

use crate::format::{CStrFormat, HexFormat};
use crate::uuid::Uuid;

pub const NV0000_CTRL_CMD_VGPU_GET_START_DATA: u32 = 0xc01;

/// See `NV0000_CTRL_VGPU_GET_START_DATA_PARAMS`
#[repr(C)]
pub struct Nv0000CtrlVgpuGetStartDataParams {
    // [u8; VM_UUID_SIZE]
    pub mdev_uuid: Uuid,
    pub config_params: [u8; 1024],
    pub qemu_pid: u32,
    pub gpu_pci_id: u32,
    pub vgpu_id: u16,
    pub gpu_pci_bdf: u32,
}

impl fmt::Debug for Nv0000CtrlVgpuGetStartDataParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Nv0000CtrlVgpuGetStartDataParams")
            .field("mdev_uuid", &format_args!("{{{}}}", self.mdev_uuid))
            .field("config_params", &CStrFormat(&self.config_params))
            .field("qemu_pid", &self.qemu_pid)
            .field("gpu_pci_id", &HexFormat(&self.gpu_pci_id))
            .field("vgpu_id", &self.vgpu_id)
            .field("gpu_pci_bdf", &self.gpu_pci_bdf)
            .finish()
    }
}

pub const NV0000_CTRL_CMD_VGPU_CREATE_DEVICE: u32 = 0xc02;

#[repr(C)]
pub struct Nv0000CtrlVgpuCreateDeviceParams {
    pub vgpu_name: Uuid,
    pub gpu_pci_id: u32,
    pub gpu_pci_bdf: u32,
    pub vgpu_type_id: u32,
    pub vgpu_id: u16,
    // R570 adds additional fields, leave them out for now for backwards compat with 16.x and 17.x
    // https://github.com/NVIDIA/open-gpu-kernel-modules/blob/570/src/common/sdk/nvidia/inc/ctrl/ctrl0000/ctrl0000vgpu.h#L94-L95
    //
    // pub gpuInstanceId: u32,
    // pub placementId: u32,
}

impl fmt::Debug for Nv0000CtrlVgpuCreateDeviceParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Nv0000CtrlVgpuCreateDeviceParams")
            .field("vgpu_name", &format_args!("{{{}}}", self.vgpu_name))
            .field("gpu_pci_id", &HexFormat(&self.gpu_pci_id))
            .field("gpu_pci_bdf", &self.gpu_pci_bdf)
            .field("vgpu_type_id", &self.vgpu_type_id)
            .field("vgpu_id", &self.vgpu_id)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::{Nv0000CtrlVgpuCreateDeviceParams, Nv0000CtrlVgpuGetStartDataParams};

    #[test]
    fn verify_sizes() {
        assert_eq!(mem::size_of::<Nv0000CtrlVgpuGetStartDataParams>(), 0x420);
        assert_eq!(mem::size_of::<Nv0000CtrlVgpuCreateDeviceParams>(), 0x20);
    }
}
