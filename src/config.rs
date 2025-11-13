// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::Deserialize;

use crate::string_number::U32;

struct Defaults;

impl Defaults {
    #[inline]
    const fn unlock() -> bool {
        true
    }

    #[inline]
    const fn unlock_migration() -> bool {
        false
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "Defaults::unlock")]
    pub unlock: bool,
    #[serde(default = "Defaults::unlock_migration")]
    pub unlock_migration: bool,
    #[serde(default)]
    pub pci_info_map: Option<HashMap<U32, PciInfoMapEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct PciInfoMapEntry {
    pub device_id: u16,
    pub sub_system_id: u16,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            unlock: Defaults::unlock(),
            unlock_migration: Defaults::unlock_migration(),
            pci_info_map: None,
        }
    }
}
