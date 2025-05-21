use crate::config::{get_global_config, RegistryConfig};
use anyhow::Result;
use reqwest::Client;
use semver::Version;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use versions::Versioning;

#[derive(Debug)]
pub struct RegistryArtifact {
    pub name: String,
    pub version: Version,
    pub url: String,
}

pub struct RegistryClient {
    config: RegistryConfig,
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryClient {
    pub fn new() -> Self {
        let config = get_global_config().registry.clone();
        Self { config }
    }

    /// Resolves an artifact by trying each registry URL in order until the artifact is found
    pub async fn resolve_artifact(
        &self,
        name: &str,
        version: &Versioning,
    ) -> Result<Option<RegistryArtifact>> {
        let client = Client::new();
        let global_config = get_global_config();

        // Convert version requirement to a concrete version string
        let version_str = version.to_string();

        // Try each registry URL in order
        for base_url in &self.config.urls {
            // Construct the artifact URL using the new structure
            let artifact_url = global_config.get_component_url(&version_str, name);
            println!("artifact_url: {}", artifact_url);

            // Try to check if the artifact exists
            match client.head(&artifact_url).send().await {
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
                        log::warn!(
                            "Registry {} returned status {} for package {} version {}",
                            base_url,
                            response.status(),
                            name,
                            version_str
                        );
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
        log::warn!(
            "Could not find package {} version {} in any registry",
            name,
            version_str
        );
        Ok(None)
    }

    /// Downloads an artifact from the registry
    pub async fn download_artifact(
        &self,
        artifact: &RegistryArtifact,
        target_path: &PathBuf,
    ) -> Result<()> {
        println!("download_artifact: {}", artifact.url);
        // Create an async client for the download
        let client = Client::new();

        // Make the request to download the artifact
        let response = client.get(&artifact.url).send().await?.error_for_status()?;

        // Get the content length for progress reporting
        let _total_size = response.content_length().unwrap_or(0);

        // Download the content
        let bytes = response.bytes().await?;

        // Write the downloaded content to the target file
        let mut file = fs::File::create(target_path.join(artifact.name.clone()))?;
        file.write_all(&bytes)?;

        log::info!(
            "Successfully downloaded {} to {}",
            artifact.name,
            target_path.display()
        );
        Ok(())
    }
}
