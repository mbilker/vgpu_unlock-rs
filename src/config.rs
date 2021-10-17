use serde::Deserialize;

struct Defaults;

impl Defaults {
    #[inline]
    const fn unlock() -> bool {
        true
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Defaults::unlock")]
    pub unlock: bool,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            unlock: Defaults::unlock(),
        }
    }
}
