// SPDX-License-Identifier: MIT

use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Defaults::unlock")]
    pub unlock: bool,
    #[serde(default = "Defaults::unlock_migration")]
    pub unlock_migration: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            unlock: Defaults::unlock(),
            unlock_migration: Defaults::unlock_migration(),
        }
    }
}
