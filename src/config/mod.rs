mod global;
mod registry;

pub use global::{GlobalConfig, GlobalConfigManager};
pub use registry::RegistryConfig;

use anyhow::Result;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

pub fn init_global_config() -> Result<()> {
    let manager = GlobalConfigManager::new()?;
    let config = manager.load_config()?;
    GLOBAL_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Failed to set global config"))?;
    Ok(())
}

pub fn get_global_config() -> &'static GlobalConfig {
    GLOBAL_CONFIG.get().expect("Global config not initialized")
}
