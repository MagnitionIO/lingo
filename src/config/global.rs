use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use dirs::home_dir;
use crate::config::registry::RegistryConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub registry: RegistryConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
        }
    }
}

pub struct GlobalConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl GlobalConfigManager {
    pub fn new() -> Result<Self> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".lingo");
        let config_file = config_dir.join("config.toml");
        
        Ok(Self {
            config_dir,
            config_file,
        })
    }

    pub fn load_config(&self) -> Result<GlobalConfig> {
        if !self.config_file.exists() {
            // Create default config if it doesn't exist
            let config = GlobalConfig::default();
            self.save_config(&config)?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&self.config_file)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_config(&self, config: &GlobalConfig) -> Result<()> {
        // Create config directory if it doesn't exist
        fs::create_dir_all(&self.config_dir)?;
        
        let contents = toml::to_string(config)?;
        fs::write(&self.config_file, contents)?;
        Ok(())
    }
} 