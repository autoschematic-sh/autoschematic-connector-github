use std::path::{Path, PathBuf};

use autoschematic_core::{connector::ResourceAddress, error_util::invalid_addr_path};

#[derive(Debug, Clone)]
pub enum GitHubResourceAddress {
    // #need(Doc, Config)
    Config,
    // #need(Doc, Repository)
    Repository { owner: String, repo: String },
    // #need(Doc, BranchProtection)
    BranchProtection { owner: String, repo: String, branch: String },
}

impl ResourceAddress for GitHubResourceAddress {
    fn to_path_buf(&self) -> PathBuf {
        match &self {
            GitHubResourceAddress::Config => PathBuf::from("github/config.ron"),
            GitHubResourceAddress::Repository { owner, repo } => PathBuf::from(format!("github/{owner}/{repo}/repository.ron")),
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => {
                PathBuf::from(format!("github/{owner}/{repo}/branches/{branch}/protection.ron"))
            }
        }
    }

    fn from_path(path: &Path) -> Result<Self, anyhow::Error> {
        let path_components: Vec<&str> = path.components().map(|s| s.as_os_str().to_str().unwrap()).collect();

        match path_components[..] {
            ["github", "config.ron"] => Ok(GitHubResourceAddress::Config),
            ["github", owner, repo, "repository.ron"] => Ok(GitHubResourceAddress::Repository {
                owner: owner.to_string(),
                repo: repo.to_string(),
            }),
            ["github", owner, repo, "branches", branch, "protection.ron"] => Ok(GitHubResourceAddress::BranchProtection {
                owner: owner.to_string(),
                repo: repo.to_string(),
                branch: branch.to_string(),
            }),
            _ => Err(invalid_addr_path(path)),
        }
    }
}
