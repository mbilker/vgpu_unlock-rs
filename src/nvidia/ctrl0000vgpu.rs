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

#[cfg(test)]
mod test {
    use std::mem;

    use super::Nv0000CtrlVgpuGetStartDataParams;

    #[test]
    fn verify_sizes() {
        assert_eq!(mem::size_of::<Nv0000CtrlVgpuGetStartDataParams>(), 0x420);
    }
}
