use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;
use versions::{Versioning, Requirement};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use crate::config::get_global_config;
use crate::config::RegistryConfig;
use semver::Version;
use std::fs;
use reqwest::Client;
use std::io::Write;
use futures_util::StreamExt;
use std::str::FromStr;

#[derive(Debug)]
pub struct RegistryArtifact {
    pub name: String,
    pub version: Version,
    pub url: String,
}

pub struct RegistryClient {
    config: RegistryConfig,
}

impl RegistryClient {
    pub fn new() -> Self {
        let config = crate::config::get_global_config().registry.clone();
        Self { config }
    }

    /// Resolves an artifact by trying each registry URL in order until the artifact is found
    pub async fn resolve_artifact(&self, name: &str, version: &Versioning) -> Result<Option<RegistryArtifact>> {
        let client = Client::new();
        
        // Convert version requirement to a concrete version string
        let version_str = version.to_string();
        
        // Try each registry URL in order
        for base_url in &self.config.urls {
            // Construct the artifact URL
            let artifact_url = format!("{}/api/v1/packages/{}/versions/{}/download", 
                base_url.trim_end_matches('/'),
                name,
                version_str);
            
            // Try to check if the artifact exists
            match client.head(&artifact_url)
                .send()
                .await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Artifact exists, parse the version for the return value
                        if let Ok(semver) = Version::parse(&version_str) {
                            return Ok(Some(RegistryArtifact {
                                name: name.to_string(),
                                version: semver,
                                url: artifact_url,
                            }));
                        }
                    } else {
                        log::warn!("Registry {} returned status {} for package {} version {}", 
                            base_url, response.status(), name, version_str);
                        continue;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to connect to registry {}: {}", base_url, e);
                    continue;
                }
            }
        }
        
        // If we get here, we couldn't find the artifact in any registry
        log::warn!("Could not find package {} version {} in any registry", name, version_str);
        Ok(None)
    }

    /// Downloads an artifact from the registry
    pub async fn download_artifact(&self, artifact: &RegistryArtifact, target_path: &PathBuf) -> Result<()> {
        // Create an async client for the download
        let client = Client::new();
        
        // Make the request to download the artifact
        let response = client.get(&artifact.url)
            .send()
            .await?
            .error_for_status()?;
        
        // Get the content length for progress reporting
        let total_size = response.content_length().unwrap_or(0);
        
        // Download the content
        let bytes = response.bytes().await?;
        
        // Write the downloaded content to the target file
        let mut file = fs::File::create(target_path)?;
        file.write_all(&bytes)?;
        
        log::info!("Successfully downloaded {} to {}", artifact.name, target_path.display());
        Ok(())
    }
} 