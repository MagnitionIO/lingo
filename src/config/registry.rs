use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub urls: Vec<String>,
    #[serde(skip)]
    pub auth_token: Option<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            urls: vec!["https://registry.lingo.io".to_string()],
            auth_token: None,
        }
    }
} 