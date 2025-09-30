use crate::config::GitHubConnectorConfig;
use anyhow::bail;
use octocrab::{Octocrab, OctocrabBuilder};

pub async fn get_client(config: Option<GitHubConnectorConfig>) -> anyhow::Result<Octocrab> {
    let Ok(token) = std::env::var("GITHUB_TOKEN") else {
        bail!("No GitHub token provided")
    };

    let mut builder = OctocrabBuilder::new().personal_token(token.to_string());

    if let Some(enterprise_url) = &config.and_then(|c| c.enterprise_url) {
        builder = builder.base_uri(enterprise_url)?;
    }

    Ok(builder.build()?)
}
