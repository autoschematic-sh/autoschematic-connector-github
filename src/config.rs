use autoschematic_core::{connector::Resource, macros::FieldTypes, util::RON};
use autoschematic_macros::FieldTypes;
use documented::{Documented, DocumentedFields};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum GithubRateLimitStrategy {
//     Conservative,
//     Aggressive,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GithubRepositoryOwner {
    User(String),
    Organization(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Documented, DocumentedFields, Clone, FieldTypes)]
#[serde(deny_unknown_fields)]
/// The primary configuration block for the GithubConnector.
pub struct GitHubConnectorConfig {
    /// A list of organization slugs that this connector should try and connect to and work with.
    pub orgs: Vec<String>,
    /// A list of user logins that this connector should manage resources under.
    pub users: Vec<String>,
    /// If using Github enterprise, the url for the enterprise
    pub enterprise_url: Option<String>,
    /// The number of requests to make in parallel. Defaults to 5.
    pub concurrent_requests: usize,
}

impl Default for GitHubConnectorConfig {
    fn default() -> Self {
        Self {
            orgs: Vec::new(),
            users: Vec::new(),
            enterprise_url: None,
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
        Ok(RON.to_string_pretty(self, PrettyConfig::default())?.into())
    }

    fn from_bytes(_addr: &impl autoschematic_core::connector::ResourceAddress, s: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(RON.from_str(str::from_utf8(s)?)?)
    }
}
