use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub urls: Vec<String>,
    #[serde(skip)]
    pub auth_token: Option<String>,
}

impl RegistryConfig {
    pub fn get_component_url(
        &self,
        base_url: &str,
        architecture: &str,
        sdk_version: &str,
        component_version: &str,
        component: &str,
    ) -> String {
        format!(
            "{}/{}/sdk{}/{}/{}",
            base_url.trim_end_matches('/'),
            architecture,
            sdk_version,
            component_version,
            component
        )
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            urls: vec!["http://localhost:8080/pulp/content/file/foo".to_string()],
            auth_token: None,
        }
    }
}
