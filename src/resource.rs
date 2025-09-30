use autoschematic_core::{
    connector::{Resource, ResourceAddress},
    error_util::invalid_addr,
    util::RON,
};
use serde::{Deserialize, Serialize};

use super::addr::GitHubResourceAddress;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitHubRepository {
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub topics: Vec<String>,
    pub private: bool,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_wiki: bool,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase_merge: bool,
    pub allow_auto_merge: bool,
    pub delete_branch_on_merge: bool,
    pub default_branch: String,
    pub archived: bool,
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RequiredStatusChecks {
    pub strict: bool,
    pub contexts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PullRequestReviewEnforcement {
    pub required_approving_review_count: u32,
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_reviews: bool,
    pub require_last_push_approval: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BranchRestrictions {
    pub users: Vec<String>,
    pub teams: Vec<String>,
    pub apps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BranchProtection {
    pub required_status_checks: Option<RequiredStatusChecks>,
    pub enforce_admins: bool,
    pub required_pull_request_reviews: Option<PullRequestReviewEnforcement>,
    pub restrictions: Option<BranchRestrictions>,
    pub required_linear_history: bool,
    pub allow_force_pushes: bool,
    pub allow_deletions: bool,
    pub block_creations: bool,
    pub required_conversation_resolution: bool,
    pub lock_branch: bool,
    pub allow_fork_syncing: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CollaboratorPermissions {
    pub pull: bool,
    pub triage: bool,
    pub push: bool,
    pub maintain: bool,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Collaborator {
    pub permissions: CollaboratorPermissions,
    pub role_name: String,
}

pub enum GitHubResource {
    Repository(GitHubRepository),
    BranchProtection(BranchProtection),
    Collaborator(Collaborator),
}

impl Resource for GitHubResource {
    fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        let pretty_config = autoschematic_core::util::PrettyConfig::default().struct_names(true);
        match self {
            GitHubResource::Repository(repo) => Ok(RON.to_string_pretty(&repo, pretty_config)?.into()),
            GitHubResource::BranchProtection(protection) => Ok(RON.to_string_pretty(&protection, pretty_config)?.into()),
            GitHubResource::Collaborator(collaborator) => Ok(RON.to_string_pretty(&collaborator, pretty_config)?.into()),
        }
    }

    fn from_bytes(addr: &impl ResourceAddress, s: &[u8]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let addr = GitHubResourceAddress::from_path(&addr.to_path_buf())?;
        let s = std::str::from_utf8(s)?;

        match addr {
            GitHubResourceAddress::Repository { .. } => Ok(GitHubResource::Repository(RON.from_str(s)?)),
            GitHubResourceAddress::BranchProtection { .. } => Ok(GitHubResource::BranchProtection(RON.from_str(s)?)),
            GitHubResourceAddress::Collaborator { .. } => Ok(GitHubResource::Collaborator(RON.from_str(s)?)),
            _ => Err(invalid_addr(&addr)),
        }
    }
}
