use crate::config::registry::RegistryConfig;
use anyhow::Result;
use dirs::home_dir;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Architecture {
    #[default]
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    AArch64,
    #[serde(rename = "arm")]
    Arm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionWrapper(#[serde(with = "version_serde")] pub Version);

mod version_serde {
    use semver::Version;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl Default for VersionWrapper {
    fn default() -> Self {
        VersionWrapper(Version::new(1, 0, 0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub registry: RegistryConfig,
    pub architecture: Architecture,
    pub sdk_version: VersionWrapper,
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
