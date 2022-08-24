// SPDX-License-Identifier: MIT

//! Credit to community members for most of the work with notable contributions by:
//!
//! - DualCoder for the original [`vgpu_unlock`](https://github.com/DualCoder/vgpu_unlock)
//! - DualCoder, snowman, Felix, Elec for vGPU profile modification at runtime
//! - NVIDIA for their open-source driver [sources](https://github.com/NVIDIA/open-gpu-kernel-modules)
//! - Arc Compute for their work on Mdev-GPU and GVM documenting more field names in the vGPU
//!   configuration structure

use std::borrow::Cow;
use std::collections::HashMap;
use std::cmp;
use std::env;
use std::fmt;
use std::fs;
use std::io::{ErrorKind, Write};
use std::mem;
use std::os::raw::{c_int, c_ulong, c_void};
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::process;
use std::str;

use ctor::ctor;
use libc::RTLD_NEXT;
use parking_lot::Mutex;
use serde::Deserialize;

mod config;
mod dump;
mod format;
mod human_number;
mod ioctl;
mod log;

use crate::config::Config;
use crate::format::{CStrFormat, HexFormat, HexFormatSlice, StraightFormat, WideCharFormat};
use crate::ioctl::_IOCWR;
use crate::log::{error, info};

static LAST_MDEV_UUID: Mutex<Option<Uuid>> = parking_lot::const_mutex(None);

#[ctor]
static CONFIG: Config = {
    match fs::read_to_string(DEFAULT_CONFIG_PATH) {
        Ok(config) => match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to decode config: {}", e);

                process::abort();
            }
        },
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                eprintln!("Failed to read config: {}", e);
            }

            Default::default()
        }
    }
};

const DEFAULT_CONFIG_PATH: &str = "/etc/vgpu_unlock/config.toml";
const DEFAULT_PROFILE_OVERRIDE_CONFIG_PATH: &str = "/etc/vgpu_unlock/profile_override.toml";

/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/kernel-open/common/inc/nv-ioctl-numbers.h
const NV_IOCTL_MAGIC: c_ulong = b'F' as _;

/// Value of the "request" argument used by `nvidia-vgpud` and `nvidia-vgpu-mgr` when calling
/// ioctl to read the PCI device ID, type, and many other things from the driver.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/nvidia/arch/nvalloc/unix/include/nv_escape.h
/// and [`nvidia_ioctl`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/nvidia/nv.c)
/// and [`__NV_IOWR`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/98553501593ef05bddcc438689ed1136f732d40a/kernel-open/common/inc/nv.h)
/// showing that `_IOCWR` is used to derive the I/O control request codes.
const NV_ESC_RM_CONTROL: c_ulong = _IOCWR::<Nvos54Parameters>(NV_IOCTL_MAGIC, 0x2a);

/// `result` is a pointer to `VgpuStart`.
const OP_READ_START_CALL: u32 = 0xc01;

/// `result` is a pointer to `uint32_t`.
const NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE: u32 = 0x800289;

/// `result` is a pointer to `uint16_t[4]`, the second element (index 1) is the device ID, the
/// forth element (index 3) is the subsystem ID.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/ctrl/ctrl2080/ctrl2080bus.h
const NV2080_CTRL_CMD_BUS_GET_PCI_INFO: u32 = 0x20801801;

/// `result` is a pointer to `VgpuConfig`.
const OP_READ_VGPU_CFG: u32 = 0xa0820102;

/// `result` is a pointer to `bool`.
const OP_READ_VGPU_MIGRATION_CAP: u32 = 0xa0810112;

/// `nvidia-vgpu-mgr` expects this value for a vGPU capable GPU.
///
/// Pulled from https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/ctrl/ctrl0080/ctrl0080gpu.h
const NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST: u32 = 3;

/// When ioctl returns success (retval >= 0) but sets the status value of the arg structure to 3
/// then `nvidia-vgpud` will sleep for a bit (first 0.1s then 1s then 10s) then issue the same
/// ioctl call again until the status differs from 3. It will attempt this for up to 24h before
/// giving up.
///
/// See https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/kernel-open/common/inc/nvstatuscodes.h
const NV_OK: u32 = 0x0;
const NV_ERR_BUSY_RETRY: u32 = 0x3;
const NV_ERR_NOT_SUPPORTED: u32 = 0x56;
const NV_ERR_OBJECT_NOT_FOUND: u32 = 0x57;

/// When issuing ioctl with `NV_ESC_RM_CONTROL` then the `argp` argument is a pointer to a
/// `NVOS54_PARAMETERS` structure like this.
///
/// See [`NVOS54_PARAMETERS`](https://github.com/NVIDIA/open-gpu-kernel-modules/blob/d8f3bcff924776518f1e63286537c3cf365289ac/src/common/sdk/nvidia/inc/nvos.h)
//#[derive(Debug)]
#[repr(C)]
struct Nvos54Parameters {
    /// Initialized prior to call.
    h_client: u32,
    /// Initialized prior to call.
    h_object: u32,
    /// Operation type, see comment below.
    cmd: u32,
    /// Pointer initialized prior to call.
    /// Pointee initialized to 0 prior to call.
    /// Pointee is written by ioctl call.
    params: *mut c_void,
    /// Size in bytes of the object referenced in `params`.
    params_size: u32,
    /// Written by ioctl call. See comment below.
    status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Uuid(u32, u16, u16, [u8; 8]);

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

#[repr(C)]
struct VgpuStart {
    uuid: Uuid,
    config_params: [u8; 1024],
    qemu_pid: u32,
    unknown_414: [u8; 12],
}

#[repr(C)]
struct VgpuConfig {
    vgpu_type: u32,
    vgpu_name: [u8; 32],
    vgpu_class: [u8; 32],
    vgpu_signature: [u8; 128],
    features: [u8; 128],
    max_instances: u32,
    num_heads: u32,
    max_resolution_x: u32,
    max_resolution_y: u32,
    max_pixels: u32,
    frl_config: u32,
    cuda_enabled: u32,
    ecc_supported: u32,
    mig_instance_size: u32,
    multi_vgpu_supported: u32,
    vdev_id: u64,
    pdev_id: u64,
    fb_length: u64,
    mappable_video_size: u64,
    fb_reservation: u64,
    encoder_capacity: u32,
    bar1_length: u64,
    frl_enable: u32,
    adapter_name: [u8; 64],
    adapter_name_unicode: [u16; 64],
    short_gpu_name_string: [u8; 64],
    licensed_product_name: [u8; 128],
    vgpu_extra_params: [u8; 1024],
}

#[derive(Deserialize)]
struct ProfileOverridesConfig<'a> {
    #[serde(borrow, default)]
    profile: HashMap<&'a str, VgpuProfileOverride<'a>>,
    #[serde(borrow, default)]
    mdev: HashMap<&'a str, VgpuProfileOverride<'a>>,
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
    #[serde(default, with = "human_number")]
    framebuffer: Option<u64>,
    #[serde(default, with = "human_number")]
    mappable_video_size: Option<u64>,
    #[serde(default, with = "human_number")]
    framebuffer_reservation: Option<u64>,
    encoder_capacity: Option<u32>,
    bar1_length: Option<u64>,
    frl_enabled: Option<u32>,
    adapter_name: Option<&'a str>,
    short_gpu_name: Option<&'a str>,
    license_type: Option<&'a str>,
}

impl fmt::Debug for VgpuStart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VgpuStart")
            .field("uuid", &format_args!("{{{}}}", self.uuid))
            .field("config_params", &CStrFormat(&self.config_params))
            .field("qemu_pid", &self.qemu_pid)
            .field("unknown_414", &StraightFormat(&self.unknown_414))
            .finish()
    }
}

impl fmt::Debug for VgpuConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VgpuConfig")
            .field("vgpu_type", &self.vgpu_type)
            .field("vgpu_name", &CStrFormat(&self.vgpu_name))
            .field("vgpu_class", &CStrFormat(&self.vgpu_class))
            .field("vgpu_signature", &HexFormatSlice(&self.vgpu_signature))
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
            .field(
                "vgpu_extra_params",
                &HexFormatSlice(&self.vgpu_extra_params[..]),
            )
            .finish()
    }
}

fn check_size(name: &str, actual_size: usize, expected_size: usize) -> bool {
    if actual_size < expected_size {
        error!(
            "Parameters size for {} was {} bytes, expected {} bytes",
            name, actual_size, expected_size
        );

        false
    } else {
        true
    }
}

/// # Safety
///
/// This is actually unsafe since `ioctl` is variadic. All the `ioctl` calls in the
/// 460.32.04 `nvidia-vgpu-mgr` and `nvidia-vgpud` binaries use only one argument.
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

    if request != NV_ESC_RM_CONTROL {
        // Not a call we care about.
        return ret;
    }

    if ret < 0 {
        // Call failed.
        return ret;
    }

    // Safety: NVIDIA's driver itself uses `sizeof` when calculating the ioctl number and so does
    // this hook so the structure passed in should be of the correct size.
    let io_data = &mut *(argp as *mut Nvos54Parameters);

    if io_data.status == NV_ERR_BUSY_RETRY {
        // Driver will try again.
        return ret;
    }

    //info!("{:x?}", io_data);

    macro_rules! check_size {
        ($name:ident, $expected_type:ty) => {
            check_size(
                stringify!($name),
                io_data.params_size as usize,
                mem::size_of::<$expected_type>(),
            )
        };
        ($name:ident, size: $expected_size:expr) => {
            check_size(
                stringify!($name),
                io_data.params_size as usize,
                $expected_size,
            )
        };
    }

    match io_data.cmd {
        NV2080_CTRL_CMD_BUS_GET_PCI_INFO
            if check_size!(NV2080_CTRL_CMD_BUS_GET_PCI_INFO, size: 8) && CONFIG.unlock =>
        {
            // Lookup address of the device and subsystem IDs.
            let devid_ptr: *mut u16 = io_data.params.add(2).cast();
            let subsysid_ptr: *mut u16 = io_data.params.add(6).cast();

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
        NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE
            if check_size!(NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE, u32) && CONFIG.unlock =>
        {
            let dev_type_ptr: *mut u32 = io_data.params.cast();

            // Set device type to vGPU capable.
            *dev_type_ptr = NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST;
        }
        OP_READ_VGPU_MIGRATION_CAP
            if check_size!(OP_READ_VGPU_MIGRATION_CAP, u8) && CONFIG.unlock_migration =>
        {
            let migration_enabled: *mut u8 = io_data.params.cast();

            *migration_enabled = 1;
        }
        _ => {}
    }

    if io_data.status == NV_OK {
        match io_data.cmd {
            OP_READ_VGPU_CFG if check_size!(OP_READ_VGPU_CFG, VgpuConfig) => {
                let config = &mut *(io_data.params as *mut VgpuConfig);
                info!("{:#?}", config);

                if !handle_profile_override(config) {
                    error!("Failed to apply profile override");
                    return -1;
                }
            }
            OP_READ_START_CALL if check_size!(OP_READ_START_CALL, VgpuStart) => {
                let config = &*(io_data.params as *const VgpuStart);
                info!("{:#?}", config);

                *LAST_MDEV_UUID.lock() = Some(config.uuid);
            }
            _ => {}
        }
    }

    if io_data.status != NV_OK {
        // Things seems to work fine even if some operations that fail result in failed assertions.
        // So here we change the status value for these cases to cleanup the logs for
        // `nvidia-vgpu-mgr`.
        if io_data.cmd == 0xa0820104 || io_data.cmd == 0x90960103 {
            io_data.status = NV_OK;
        } else {
            error!("cmd: 0x{:x} failed.", io_data.cmd);
        }
    }

    // Workaround for some Maxwell cards not supporting reading inforom.
    if io_data.cmd == 0x2080014b && io_data.status == NV_ERR_NOT_SUPPORTED {
        io_data.status = NV_ERR_OBJECT_NOT_FOUND;
    }

    ret
}

pub fn from_c_str(value: &[u8]) -> Cow<'_, str> {
    let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());

    String::from_utf8_lossy(&value[..len])
}

fn handle_profile_override(config: &mut VgpuConfig) -> bool {
    let config_path = match env::var_os("VGPU_UNLOCK_PROFILE_OVERRIDE_CONFIG_PATH") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_PROFILE_OVERRIDE_CONFIG_PATH),
    };

    let config_overrides = match fs::read_to_string(&config_path) {
        Ok(data) => data,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                error!("Config file '{}' not found", config_path.display());
                return true;
            }

            error!("Failed to read '{}': {}", config_path.display(), e);
            return false;
        }
    };

    let config_overrides: ProfileOverridesConfig = match toml::from_str(&config_overrides) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to decode config: {}", e);
            return false;
        }
    };

    let vgpu_type = format!("nvidia-{}", config.vgpu_type);
    let mdev_uuid = LAST_MDEV_UUID.lock().take();

    if let Some(config_override) = config_overrides.profile.get(vgpu_type.as_str()) {
        info!("Applying profile {} overrides", vgpu_type);

        if !apply_profile_override(config, &vgpu_type, config_override) {
            return false;
        }
    }
    if let Some(mdev_uuid) = mdev_uuid.map(|uuid| uuid.to_string()) {
        if let Some(config_override) = config_overrides.mdev.get(mdev_uuid.as_str()) {
            info!("Applying mdev UUID {} profile overrides", mdev_uuid);

            if !apply_profile_override(config, &vgpu_type, config_override) {
                return false;
            }
        }
    }

    true
}

fn apply_profile_override(
    config: &mut VgpuConfig,
    vgpu_type: &str,
    config_override: &VgpuProfileOverride,
) -> bool {
    macro_rules! handle_copy_overrides {
        ($source_field:ident => $target_field:ident) => {
            if let Some(value) = config_override.$source_field {
                info!(
                    "Patching {}/{}: {} -> {}",
                    vgpu_type,
                    stringify!($target_field),
                    config.$target_field,
                    value
                );

                config.$target_field = value;
            }
        };
        ($field:ident) => {
            handle_copy_overrides!($field => $field);
        };
        ($($source_field:ident $(=> $target_field:ident)?),*$(,)?) => {
            $(
                handle_copy_overrides!($source_field $(=> $target_field)?);
            )*
        };
    }
    macro_rules! handle_bool_overrides {
        ($source_field:ident => $target_field:ident) => {
            if let Some(value) = config_override.$source_field {
                let value = cmp::max(cmp::min(value, 0), 1);

                info!(
                    "Patching {}/{}: {} -> {}",
                    vgpu_type,
                    stringify!($target_field),
                    config.$target_field,
                    value
                );

                config.$target_field = value;
            }
        };
        ($field:ident) => {
            handle_bool_overrides!($field => $field);
        };
        ($($source_field:ident $(=> $target_field:ident)?),*$(,)?) => {
            $(
                handle_bool_overrides!($source_field $(=> $target_field)?);
            )*
        };
    }
    macro_rules! handle_str_overrides {
        ($source_field:ident => $target_field:ident) => {
            if let Some(value) = config_override.$source_field {
                let value_bytes = value.as_bytes();

                // Use `len - 1` to account for the required NULL terminator.
                if value_bytes.len() > config.$target_field.len() - 1 {
                    error!(
                        "Patching {}/{}: value '{}' is too long",
                        vgpu_type,
                        stringify!($target_field),
                        value
                    );

                    return false;
                } else {
                    info!(
                        "Patching {}/{}: '{}' -> '{}'",
                        vgpu_type,
                        stringify!($target_field),
                        from_c_str(&config.$target_field),
                        value
                    );

                    // Zero out the field first.
                    // (`fill` was stabilized in Rust 1.50, but Debian Bullseye ships with 1.48)
                    for v in config.$target_field.iter_mut() {
                        *v = 0;
                    }

                    // Write the string bytes.
                    let _ = config.$target_field[..].as_mut().write_all(value_bytes);
                }
            }
        };
        ($field:ident) => {
            handle_str_overrides!($field => $field);
        };
        ($($source_field:ident $(=> $target_field:ident)?),*$(,)?) => {
            $(
                handle_str_overrides!($source_field $(=> $target_field)?);
            )*
        };
    }
    macro_rules! handle_wide_str_overrides {
        ($source_field:ident => $target_field:ident) => {
            if let Some(value) = config_override.$source_field {
                // Use `len - 1` to account for the required NULL terminator.
                if value.encode_utf16().count() > config.$target_field.len() - 1 {
                    error!(
                        "Patching {}/{}: value '{}' is too long",
                        vgpu_type,
                        stringify!($target_field),
                        value
                    );

                    return false;
                } else {
                    info!(
                        "Patching {}/{}: '{}' -> '{}'",
                        vgpu_type,
                        stringify!($target_field),
                        WideCharFormat(&config.$target_field),
                        value
                    );

                    // Zero out the field first.
                    // (`fill` was stabilized in Rust 1.50, but Debian Bullseye ships with 1.48)
                    for v in config.$target_field.iter_mut() {
                        *v = 0;
                    }

                    // Write the string bytes.
                    for (v, ch) in config.$target_field[..]
                        .iter_mut()
                        .zip(value.encode_utf16().chain(Some(0)))
                    {
                        *v = ch;
                    }
                }
            }
        };
        ($field:ident) => {
            handle_wide_str_overrides!($field => $field);
        };
        ($($source_field:ident $(=> $target_field:ident)?),*$(,)?) => {
            $(
                handle_wide_str_overrides!($source_field $(=> $target_field)?);
            )*
        };
    }

    // While the following could be done with two statements. I wanted the log statements to be in
    // field order.

    handle_copy_overrides! {
        gpu_type => vgpu_type,
    }
    handle_str_overrides! {
        card_name => vgpu_name,
        vgpu_type => vgpu_class,
        features,
    }
    handle_copy_overrides! {
        max_instances,
        num_displays => num_heads,
        display_width => max_resolution_x,
        display_height => max_resolution_y,
        max_pixels,
        frl_config,
    }
    handle_bool_overrides! {
        cuda_enabled,
        ecc_supported,
    }
    handle_copy_overrides! {
        mig_instance_size,
    }
    handle_bool_overrides! {
        multi_vgpu_supported,
    }
    handle_copy_overrides! {
        pci_id => vdev_id,
        pci_device_id => pdev_id,
        framebuffer => fb_length,
        mappable_video_size,
        framebuffer_reservation => fb_reservation,
        encoder_capacity,
        bar1_length,
    }
    handle_bool_overrides! {
        frl_enabled => frl_enable,
    }
    handle_str_overrides! {
        adapter_name,
    }
    handle_wide_str_overrides! {
        adapter_name => adapter_name_unicode,
    }
    handle_str_overrides! {
        short_gpu_name => short_gpu_name_string,
        license_type => licensed_product_name,
    }

    true
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::{VgpuConfig, VgpuStart};

    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<VgpuStart>(), 0x420);
        assert_eq!(mem::size_of::<VgpuConfig>(), 0x730);
    }
}
