// SPDX-License-Identifier: MIT

//! Credit to community members for most of the work with notable contributions by:
//!
//! - DualCoder for the original [`vgpu_unlock`](https://github.com/DualCoder/vgpu_unlock)
//! - DualCoder, snowman, Felix, Elec for vGPU profile modification at runtime
//! - NVIDIA for their open-source driver [sources](https://github.com/NVIDIA/open-gpu-kernel-modules)
//! - Arc Compute for their work on Mdev-GPU and GVM documenting more field names in the vGPU
//!   configuration structure

use std::borrow::Cow;
use std::cmp;
use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::mem;
use std::os::raw::{c_int, c_ulong, c_void};
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use std::process;
use std::str;

use consts::DEFAULT_PROFILE_OVERRIDE_CONFIG_PATH;
use ctor::ctor;
use libc::RTLD_NEXT;
use parking_lot::Mutex;
use structs::ProfileOverridesConfig;
use structs::Uuid;
use structs::VgpuConfigLike;
use structs::VgpuProfileOverride;

mod config;
mod consts;
mod dump;
mod format;
mod human_number;
mod ioctl;
mod log;
mod structs;

use crate::config::Config;
use crate::consts::DEFAULT_CONFIG_PATH;
use crate::consts::NV0000_CTRL_CMD_VGPU_GET_START_DATA;
use crate::consts::NV0080_CTRL_CMD_GPU_GET_VIRTUALIZATION_MODE;
use crate::consts::NV0080_CTRL_GPU_VIRTUALIZATION_MODE_HOST;
use crate::consts::NV2080_CTRL_CMD_BUS_GET_PCI_INFO;
use crate::consts::NV2080_CTRL_CMD_GPU_GET_INFOROM_OBJECT_VERSION;
use crate::consts::NV9096_CTRL_CMD_GET_ZBC_CLEAR_TABLE;
use crate::consts::NVA081_CTRL_CMD_VGPU_CONFIG_GET_VGPU_TYPE_INFO;
use crate::consts::NV_ERR_BUSY_RETRY;
use crate::consts::NV_ERR_NOT_SUPPORTED;
use crate::consts::NV_ERR_OBJECT_NOT_FOUND;
use crate::consts::NV_ESC_RM_CONTROL;
use crate::consts::NV_OK;
use crate::consts::OP_READ_VGPU_CFG;
use crate::consts::OP_READ_VGPU_MIGRATION_CAP;
use crate::format::WideCharFormat;
use crate::log::{error, info};
use crate::structs::Nv0000CtrlVgpuGetStartDataParams;
use crate::structs::Nv2080CtrlBusGetPciInfoParams;
use crate::structs::Nva081CtrlVgpuConfigGetVgpuTypeInfoParams;
use crate::structs::Nvos54Parameters;
use crate::structs::VgpuConfig;

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

fn check_size(name: &str, actual_size: usize, expected_size: usize) -> bool {
    if actual_size != expected_size {
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
            if check_size!(NV2080_CTRL_CMD_BUS_GET_PCI_INFO, size: 16) && CONFIG.unlock =>
        {
            let params = &mut *io_data.params.cast::<Nv2080CtrlBusGetPciInfoParams>();

            let orig_devid = params.pci_device_id;
            let orig_subsysid = params.pci_sub_system_id;

            let actual_devid = orig_devid & 0xffff;
            let actual_subsysid = orig_subsysid & 0xffff;

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

            params.pci_device_id = (orig_devid & 0xffff0000) | spoofed_devid;
            params.pci_sub_system_id = (orig_subsysid & 0xffff0000) | spoofed_subsysid;
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
            NVA081_CTRL_CMD_VGPU_CONFIG_GET_VGPU_TYPE_INFO
                if check_size!(OP_READ_VGPU_CFG2, Nva081CtrlVgpuConfigGetVgpuTypeInfoParams) =>
            {
                let config = &mut *io_data
                    .params
                    .cast::<Nva081CtrlVgpuConfigGetVgpuTypeInfoParams>();
                info!("{:#?}", config);

                if !handle_profile_override(&mut config.vgpu_type_info) {
                    error!("Failed to apply profile override");
                    return -1;
                }
            }
            NV0000_CTRL_CMD_VGPU_GET_START_DATA
                if check_size!(OP_READ_START_CALL, Nv0000CtrlVgpuGetStartDataParams) =>
            {
                let config = &*(io_data.params as *const Nv0000CtrlVgpuGetStartDataParams);
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
        if io_data.cmd == 0xa0820104 || io_data.cmd == NV9096_CTRL_CMD_GET_ZBC_CLEAR_TABLE {
            io_data.status = NV_OK;
        } else {
            error!("cmd: 0x{:x} failed.", io_data.cmd);
        }
    }

    // Workaround for some Maxwell cards not supporting reading inforom.
    if io_data.cmd == NV2080_CTRL_CMD_GPU_GET_INFOROM_OBJECT_VERSION
        && io_data.status == NV_ERR_NOT_SUPPORTED
    {
        io_data.status = NV_ERR_OBJECT_NOT_FOUND;
    }

    ret
}

pub fn from_c_str(value: &[u8]) -> Cow<'_, str> {
    let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());

    String::from_utf8_lossy(&value[..len])
}

fn load_overrides() -> Result<String, bool> {
    let config_path = match env::var_os("VGPU_UNLOCK_PROFILE_OVERRIDE_CONFIG_PATH") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_PROFILE_OVERRIDE_CONFIG_PATH),
    };

    let config_overrides = match fs::read_to_string(&config_path) {
        Ok(data) => data,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                error!("Config file '{}' not found", config_path.display());
                return Err(true);
            }

            error!("Failed to read '{}': {}", config_path.display(), e);
            return Err(false);
        }
    };

    Ok(config_overrides)
}

fn handle_profile_override<C: VgpuConfigLike>(config: &mut C) -> bool {
    let config_overrides = match load_overrides() {
        Ok(overrides) => overrides,
        Err(e) => return e,
    };

    let config_overrides: ProfileOverridesConfig = match toml::from_str(&config_overrides) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to decode config: {}", e);
            return false;
        }
    };

    let vgpu_type = format!("nvidia-{}", config.vgpu_type());
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

fn apply_profile_override<C: VgpuConfigLike>(
    config: &mut C,
    vgpu_type: &str,
    config_override: &VgpuProfileOverride,
) -> bool {
    macro_rules! patch_msg {
        ($target_field:ident, $value:expr) => {
            info!(
                "Patching {}/{}: {} -> {}",
                vgpu_type,
                stringify!($target_field),
                config.$target_field(),
                $value
            );
        };
        ($target_field:ident, $preprocess:ident, $value:expr) => {
            info!(
                "Patching {}/{}: {} -> {}",
                vgpu_type,
                stringify!($target_field),
                $preprocess(config.$target_field()),
                $value
            );
        };
    }
    macro_rules! error_too_long {
        ($target_field:ident, $value:expr) => {
            error!(
                "Patching {}/{}: value '{}' is too long",
                vgpu_type,
                stringify!($target_field),
                $value
            );

            return false;
        };
    }

    macro_rules! handle_override {
        // Override entrypoint when the same field name is used as the source and target without
        // an explicit `=>`.
        (
            class: $class:ident,
            source_field: $field:ident,
        ) => {
            handle_override! {
                class: $class,
                source_field: $field,
                target_field: $field,
            }
        };

        // Override entrypoint when both the source and target field names are defined explicitly.
        (
            class: $class:ident,
            source_field: $source_field:ident,
            target_field: $target_field:ident,
        ) => {
            if let Some(value) = config_override.$source_field {
                handle_override! {
                    class: $class,
                    value: value,
                    source_field: $source_field,
                    target_field: $target_field,
                }
            }
        };

        // The following are override handlers for each field class type (`bool`, `copy`, `str`,
        // and `wide_str`).
        (
            class: bool,
            value: $value:ident,
            source_field: $source_field:ident,
            target_field: $target_field:ident,
        ) => {
            let $value = cmp::max(cmp::min($value, 1), 0);

            patch_msg!($target_field, $value);

            *config.$target_field() = $value;
        };
        (
            class: copy,
            value: $value:ident,
            source_field: $source_field:ident,
            target_field: $target_field:ident,
        ) => {
            patch_msg!($target_field, $value);

            *config.$target_field() = $value;
        };
        (
            class: str,
            value: $value:ident,
            source_field: $source_field:ident,
            target_field: $target_field:ident,
        ) => {
            let value_bytes = $value.as_bytes();

            // Use `len - 1` to account for the required NULL terminator.
            if value_bytes.len() > config.$target_field().len() - 1 {
                error_too_long!($target_field, $value);
            } else {
                patch_msg!($target_field, from_c_str, $value);

                // Zero out the field first.
                // (`fill` was stabilized in Rust 1.50, but Debian Bullseye ships with 1.48)
                for v in config.$target_field().iter_mut() {
                    *v = 0;
                }

                // Write the string bytes.
                let _ = config.$target_field()[..].as_mut().write_all(value_bytes);
            }
        };
        (
            class: wide_str,
            value: $value:ident,
            source_field: $source_field:ident,
            target_field: $target_field:ident,
        ) => {
            // Use `len - 1` to account for the required NULL terminator.
            if $value.encode_utf16().count() > config.$target_field().len() - 1 {
                error_too_long!($target_field, $value);
            } else {
                patch_msg!($target_field, WideCharFormat, $value);

                // Zero out the field first.
                // (`fill` was stabilized in Rust 1.50, but Debian Bullseye ships with 1.48)
                for v in config.$target_field().iter_mut() {
                    *v = 0;
                }

                // Write the string bytes.
                for (v, ch) in config.$target_field()[..]
                    .iter_mut()
                    .zip($value.encode_utf16().chain(Some(0)))
                {
                    *v = ch;
                }
            }
        };
    }
    macro_rules! handle_overrides {
        (
            $($class:ident: [
                $($source_field:ident $(=> $target_field:ident)?),*$(,)?
            ]),*$(,)?
        ) => {
            $(
                $(
                    handle_override! {
                        class: $class,
                        source_field: $source_field,
                        $(target_field: $target_field,)?
                    }
                )*
            )*
        };
    }

    // While the following could be done with fewer branches, I wanted the log statements to be in
    // field order.

    handle_overrides! {
        copy: [
            gpu_type => vgpu_type,
        ],
        str: [
            card_name => vgpu_name,
            vgpu_type => vgpu_class,
            features,
        ],
        copy: [
            max_instances,
            num_displays => num_heads,
            display_width => max_resolution_x,
            display_height => max_resolution_y,
            max_pixels,
            frl_config,
        ],
        bool: [
            cuda_enabled,
            ecc_supported,
        ],
        copy: [
            mig_instance_size,
        ],
        bool: [
            multi_vgpu_supported,
        ],
        copy: [
            pci_id => vdev_id,
            pci_device_id => pdev_id,
            framebuffer => fb_length,
            mappable_video_size,
            framebuffer_reservation => fb_reservation,
            encoder_capacity,
            bar1_length,
        ],
        bool: [
            frl_enabled => frl_enable,
        ],
        str: [
            adapter_name,
        ],
        wide_str: [
            adapter_name => adapter_name_unicode,
        ],
        str: [
            short_gpu_name => short_gpu_name_string,
            license_type => licensed_product_name,
        ],
    }

    true
}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::structs::{
        Nv0000CtrlVgpuGetStartDataParams, Nva081CtrlVgpuConfigGetVgpuTypeInfoParams,
        Nva081CtrlVgpuInfo, VgpuConfig,
    };

    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<Nv0000CtrlVgpuGetStartDataParams>(), 0x420);
        assert_eq!(mem::size_of::<VgpuConfig>(), 0x730);
    }

    #[test]
    fn verify_vgpu_config2_size() {
        assert_eq!(std::mem::size_of::<Nva081CtrlVgpuInfo>(), 0x1358);
    }

    #[test]
    fn verify_load_vgpu_config2_size() {
        assert_eq!(
            std::mem::size_of::<Nva081CtrlVgpuConfigGetVgpuTypeInfoParams>(),
            0x1360
        );
    }
}
