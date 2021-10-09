// SPDX-License-Identifier: MIT

//! Credit to community members for most of the work with notable contributions by:
//!
//! - DualCoder for the original [`vgpu_unlock`](https://github.com/DualCoder/vgpu_unlock)
//! - DualCoder, snowman, Felix, Elec for vGPU profile modification at runtime

use std::borrow::Cow;
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::mem;
use std::os::raw::{c_int, c_ulong, c_void};
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::str;

use libc::RTLD_NEXT;
use serde::Deserialize;

mod format;
mod log;

use crate::format::{CStrFormat, HexFormat, StraightFormat};
use crate::log::{error, info};

/// Value of the "request" argument used by `nvidia-vgpud` and `nvidia-vgpu-mgr` when calling
/// ioctl to read the PCI device ID and type (and possibly other things) from the GPU.
const REQ_QUERY_GPU: c_ulong = 0xc020462a;

/// `result` is a pointer to `uint32_t`.
const OP_READ_DEV_TYPE: u32 = 0x800289;

/// `result` is a pointer to `uint16_t[4]`, the second element (index 1) is the device ID, the
/// forth element (index 3) is the subsystem ID.
const OP_READ_PCI_ID: u32 = 0x20801801;

/// `result` is a pointer to `VgpuConfig`.
const OP_READ_VGPU_CFG: u32 = 0xa0820102;

/// `nvidia-vgpu-mgr` expects this value for a vGPU capable GPU.
const DEV_TYPE_VGPU_CAPABLE: u32 = 3;

/// When ioctl returns success (retval >= 0) but sets the status value of the arg structure to 3
/// then `nvidia-vgpud` will sleep for a bit (first 0.1s then 1s then 10s) then issue the same
/// ioctl call again until the status differs from 3. It will attempt this for up to 24h before
/// giving up.
const STATUS_OK: u32 = 0;
const STATUS_TRY_AGAIN: u32 = 3;

/// When issuing ioctl with REQ_QUERY_GPU then the `argp` argument is a pointer to a structure
/// like this
//#[derive(Debug)]
#[repr(C)]
struct Request {
    /// Initialized prior to call.
    unknown_1: u32,
    /// Initialized prior to call.
    unknown_2: u32,
    /// Operation type, see comment below.
    op_type: u32,
    /// Pointer initialized prior to call.
    /// Pointee initialized to 0 prior to call.
    /// Pointee is written by ioctl call.
    result: *mut c_void,
    /// Set to 0x10 for READ_PCI_ID and set to 4 for
    /// READ_DEV_TYPE prior to call.
    unknown_4: u32,
    /// Written by ioctl call. See comment below.
    status: u32,
}

#[repr(C)]
struct VgpuConfig {
    gpu_type: u32,
    card_name: [u8; 32],
    vgpu_type: [u8; 160],
    features: [u8; 128],
    max_instances: u32,
    num_displays: u32,
    display_width: u32,
    display_height: u32,
    max_pixels: u32,
    frl_config: u32,
    cuda_enabled: u32,
    ecc_supported: u32,
    mig_instance_size: u32,
    multi_vgpu_supported: u32,
    pci_id: u64,
    pci_device_id: u64,
    framebuffer: u64,
    mappable_video_size: u64,
    framebuffer_reservation: u64,
    encoder_capacity: u64,
    bar1_length: u64,
    frl_enabled: u32,
    blob: [u8; 256],
    license_type: [u8; 1156],
}

#[derive(Deserialize)]
struct VgpuProfileOverride<'a> {
    gpu_type: Option<u32>,
    card_name: Option<&'a str>,
    vgpu_type: Option<&'a str>,
    features: Option<&'a str>,
    max_instances: Option<u32>,
    num_displays: Option<u32>,
    display_width: Option<u32>,
    display_height: Option<u32>,
    max_pixels: Option<u32>,
    frl_config: Option<u32>,
    cuda_enabled: Option<u32>,
    ecc_supported: Option<u32>,
    mig_instance_size: Option<u32>,
    multi_vgpu_supported: Option<u32>,
    pci_id: Option<u64>,
    pci_device_id: Option<u64>,
    framebuffer: Option<u64>,
    mappable_video_size: Option<u64>,
    framebuffer_reservation: Option<u64>,
    encoder_capacity: Option<u64>,
    bar1_length: Option<u64>,
    frl_enabled: Option<u32>,
    license_type: Option<&'a str>,
}

impl fmt::Debug for VgpuConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VgpuConfig")
            .field("gpu_type", &self.gpu_type)
            .field("card_name", &CStrFormat(&self.card_name))
            .field("vgpu_type", &CStrFormat(&self.vgpu_type))
            .field("features", &CStrFormat(&self.features))
            .field("max_instances", &self.max_instances)
            .field("num_displays", &self.num_displays)
            .field("display_width", &self.display_width)
            .field("display_height", &self.display_height)
            .field("max_pixels", &self.max_pixels)
            .field("frl_config", &self.frl_config)
            .field("cuda_enabled", &self.cuda_enabled)
            .field("ecc_supported", &self.ecc_supported)
            .field("mig_instance_size", &self.mig_instance_size)
            .field("multi_vgpu_supported", &self.multi_vgpu_supported)
            .field("pci_id", &HexFormat(self.pci_id))
            .field("pci_device_id", &HexFormat(self.pci_device_id))
            .field("framebuffer", &HexFormat(self.framebuffer))
            .field("mappable_video_size", &HexFormat(self.mappable_video_size))
            .field(
                "framebuffer_reservation",
                &HexFormat(self.framebuffer_reservation),
            )
            .field("encoder_capacity", &HexFormat(self.encoder_capacity))
            .field("bar1_length", &HexFormat(self.bar1_length))
            .field("blob", &StraightFormat(&self.blob))
            .field("license_type", &CStrFormat(&self.license_type))
            .finish()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: RawFd, request: c_ulong, argp: *mut c_void) -> c_int {
    static mut IOCTL_FN_PTR: Option<unsafe extern "C" fn(RawFd, c_ulong, ...) -> c_int> = None;

    //info!("ioctl({}, {}, {:?})", fd, request, data);

    let next_ioctl = match IOCTL_FN_PTR {
        Some(func) => func,
        None => {
            let next_ioctl = mem::transmute(libc::dlsym(RTLD_NEXT, b"ioctl\0".as_ptr() as _));

            IOCTL_FN_PTR = mem::transmute(next_ioctl);

            next_ioctl
        }
    };

    let ret = next_ioctl(fd, request, argp);

    if request != REQ_QUERY_GPU {
        // Not a call we care about.
        return ret;
    }

    if ret < 0 {
        // Call failed.
        return ret;
    }

    let io_data = &mut *(argp as *mut Request);

    if io_data.status == STATUS_TRY_AGAIN {
        // Driver will try again.
        return ret;
    }

    //info!("{:x?}", io_data);

    match io_data.op_type {
        OP_READ_PCI_ID => {
            // Lookup address of the device and subsystem IDs.
            let devid_ptr: *mut u16 = io_data.result.add(2).cast();
            let subsysid_ptr: *mut u16 = io_data.result.add(6).cast();

            let actual_devid = *devid_ptr;
            let actual_subsysid = *subsysid_ptr;

            let (spoofed_devid, spoofed_subsysid) = match actual_devid {
                // Maxwell
                0x1340..=0x13bd | 0x174d..=0x179c => {
                    // Tesla M10
                    (0x13bd, 0x1160)
                }
                // Maxwell 2.0
                0x13c0..=0x1436 | 0x1617..=0x1667 | 0x17c2..=0x17fd => {
                    // Tesla M60
                    (0x13f2, actual_subsysid)
                }
                // Pascal
                0x15f0 | 0x15f1 | 0x1b00..=0x1d56 | 0x1725..=0x172f => {
                    // Tesla P40
                    (0x1b38, actual_subsysid)
                }
                // GV100 Volta
                //
                // 0x1d81 = TITAN V
                // 0x1dba = Quadro GV100 32GB
                0x1d81 | 0x1dba => {
                    // Tesla V100 32GB PCIE
                    (0x1db6, actual_subsysid)
                }
                // Turing
                0x1e02..=0x1ff9 | 0x2182..=0x21d1 => {
                    // Quadro RTX 6000
                    (0x1e30, 0x12ba)
                }
                // Ampere
                0x2200..=0x2600 => {
                    // RTX A6000
                    (0x2230, actual_subsysid)
                }
                _ => (actual_devid, actual_subsysid),
            };

            *devid_ptr = spoofed_devid;
            *subsysid_ptr = spoofed_subsysid;
        }
        OP_READ_DEV_TYPE => {
            let dev_type_ptr: *mut u32 = io_data.result.cast();

            // Set device type to vGPU capable.
            *dev_type_ptr = DEV_TYPE_VGPU_CAPABLE;
        }
        OP_READ_VGPU_CFG => {
            let config = &mut *(io_data.result as *mut VgpuConfig);
            info!("{:#?}", config);

            if !apply_profile_override(config) {
                error!("Failed to apply profile override");
                return -1;
            }
        }
        _ => {}
    }

    if io_data.status != STATUS_OK {
        // Things seems to work fine even if some operations that fail result in failed assertions.
        // So here we change the status value for these cases to cleanup the logs for
        // `nvidia-vgpu-mgr`.
        if io_data.op_type == 0xa0820104 || io_data.op_type == 0x90960103 {
            io_data.status = STATUS_OK;
        } else {
            error!("op_type: 0x{:x} failed.", io_data.op_type);
        }
    }

    // Workaround for some Maxwell cards not supporting reading inforom.
    if io_data.op_type == 0x2080014b && io_data.status == 0x56 {
        io_data.status = 0x57;
    }

    ret
}

pub fn from_c_str<'a>(value: &'a [u8]) -> Cow<'a, str> {
    let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());

    String::from_utf8_lossy(&value[..len])
}

fn apply_profile_override(config: &mut VgpuConfig) -> bool {
    const DEFAULT_CONFIG_PATH: &'static str = "/etc/vgpu_unlock/profile_override.toml";

    let config_path = match env::var_os("VGPU_UNLOCK_PROFILE_OVERRIDE_CONFIG_PATH") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_CONFIG_PATH),
    };
    let config_overrides = match fs::read_to_string(&config_path) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read '{}': {}", config_path.display(), e);
            return false;
        }
    };
    let config_override: VgpuProfileOverride = match toml::from_str(&config_overrides) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to decode config: {}", e);
            return false;
        }
    };

    let gpu_type = config.gpu_type;

    macro_rules! handle_copy_overrides {
        ($field:ident) => {
            if let Some(value) = config_override.$field {
                info!(
                    "Patching nvidia-{}/{}: {} -> {}",
                    gpu_type,
                    stringify!($field),
                    config.$field,
                    value
                );

                config.$field = value;
            }
        };
        ($($field:ident),*$(,)?) => {
            $(
                handle_copy_overrides!($field);
            )*
        };
    }
    macro_rules! handle_str_overrides {
        ($field:ident) => {
            if let Some(value) = config_override.$field {
                let value_bytes = value.as_bytes();

                // Use `len - 1` to account for the required NULL terminator.
                if value_bytes.len() > config.$field.len() - 1 {
                    error!(
                        "Patching nvidia-{}/{}: value '{}' is too long",
                        gpu_type,
                        stringify!($field),
                        value
                    );

                    return false;
                } else {
                    info!(
                        "Patching nvidia-{}/{}: '{}' -> '{}'",
                        gpu_type,
                        stringify!($field),
                        from_c_str(&config.$field),
                        value
                    );

                    // Zero out the field first.
                    config.$field.fill(0);

                    // Write the string bytes.
                    let _ = config.$field[..].as_mut().write_all(value_bytes);
                }
            }
        };
        ($($field:ident),*$(,)?) => {
            $(
                handle_str_overrides!($field);
            )*
        };
    }

    // While the following could be done with two statements. I wanted the log statements to be in
    // field order.

    handle_copy_overrides! {
        gpu_type,
    }
    handle_str_overrides! {
        card_name,
        vgpu_type,
        features,
    }
    handle_copy_overrides! {
        max_instances,
        num_displays,
        display_width,
        display_height,
        max_pixels,
        frl_config,
        cuda_enabled,
        ecc_supported,
        mig_instance_size,
        multi_vgpu_supported,
        pci_id,
        pci_device_id,
        framebuffer,
        mappable_video_size,
        framebuffer_reservation,
        encoder_capacity,
        bar1_length,
        frl_enabled,
    }
    handle_str_overrides! {
        license_type,
    }

    true
}
