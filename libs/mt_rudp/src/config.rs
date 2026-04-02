// src/config.rs
pub struct Config {
    pub some_setting: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config { some_setting: 42 }
    }
}