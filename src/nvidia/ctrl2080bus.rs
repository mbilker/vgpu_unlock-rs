///! Sourced from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/5f40a5aee5ef9c92085836bf5b5a9056174f07f1/src/common/sdk/nvidia/inc/ctrl/ctrl2080/ctrl2080bus.h

pub const NV2080_CTRL_CMD_BUS_GET_PCI_INFO: u32 = 0x20801801;

/// See `NV2080_CTRL_BUS_GET_PCI_INFO_PARAMS`
//#[derive(Debug)]
#[repr(C)]
pub struct Nv2080CtrlBusGetPciInfoParams {
    pub pci_device_id: u32,
    pub pci_sub_system_id: u32,
    pub pci_revision_id: u32,
    pub pci_ext_device_id: u32,
}
