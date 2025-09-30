use std::path::{Path, PathBuf};

use autoschematic_core::{connector::ResourceAddress, error_util::invalid_addr_path};

#[derive(Debug, Clone)]
pub enum GitHubResourceAddress {
    Config,
    Repository { owner: String, repo: String },
    BranchProtection { owner: String, repo: String, branch: String },
    Collaborator { owner: String, repo: String, username: String },
}

impl ResourceAddress for GitHubResourceAddress {
    fn to_path_buf(&self) -> PathBuf {
        match &self {
            GitHubResourceAddress::Config => PathBuf::from("github/config.ron"),
            GitHubResourceAddress::Repository { owner, repo } => PathBuf::from(format!("github/{owner}/{repo}/repository.ron")),
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => {
                PathBuf::from(format!("github/{owner}/{repo}/branches/{branch}/protection.ron"))
            }
            GitHubResourceAddress::Collaborator { owner, repo, username } => {
                PathBuf::from(format!("github/{owner}/{repo}/collaborators/{username}.ron"))
            }
        }
    }

    fn from_path(path: &Path) -> Result<Self, anyhow::Error> {
        let path_components: Vec<&str> = path.components().map(|s| s.as_os_str().to_str().unwrap()).collect();

        match path_components[..] {
            ["github", owner, repo, "repository.ron"] => {
                Ok(GitHubResourceAddress::Repository {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                })
            }
            ["github", owner, repo, "branches", branch, "protection.ron"] => {
                Ok(GitHubResourceAddress::BranchProtection {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    branch: branch.to_string(),
                })
            }
            ["github", owner, repo, "collaborators", username] if username.ends_with(".ron") => {
                let username = username.strip_suffix(".ron").unwrap().to_string();
                Ok(GitHubResourceAddress::Collaborator {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    username,
                })
            }
            _ => Err(invalid_addr_path(path)),
        }
    }
}
