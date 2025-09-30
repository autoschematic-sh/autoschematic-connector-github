use autoschematic_core::{connector::Resource, util::RON};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GithubRateLimitStrategy {
    Conservative,
    Aggressive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GithubRepositoryOwner {
    User(String),
    Organization(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GitHubConnectorConfig {
    pub owners: Vec<String>,
    pub enterprise_url: Option<String>,
    pub rate_limit_strategy: GithubRateLimitStrategy,
    pub concurrent_requests: u32,
}

impl Default for GitHubConnectorConfig {
    fn default() -> Self {
        Self {
            owners: Vec::new(),
            enterprise_url: None,
            rate_limit_strategy: GithubRateLimitStrategy::Conservative,
            concurrent_requests: 5,
        }
    }
}

impl GitHubConnectorConfig {
    pub fn try_load(prefix: &Path) -> anyhow::Result<Option<Self>> {
        let config_path = prefix.join("github").join("config.ron");

        if !config_path.exists() {
            return Ok(None);
        }

        let config_str = std::fs::read_to_string(&config_path)?;
        let config: GitHubConnectorConfig = RON.from_str(&config_str)?;
        Ok(Some(config))
    }
}

impl Resource for GitHubConnectorConfig {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(RON.to_string(self)?.into())
    }

    fn from_bytes(_addr: &impl autoschematic_core::connector::ResourceAddress, s: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(RON.from_str(str::from_utf8(s)?)?)
    }
}
