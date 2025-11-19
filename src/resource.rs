use std::collections::HashMap;

use autoschematic_core::{
    connector::{Resource, ResourceAddress},
    error_util::invalid_addr,
    macros::FieldTypes,
    util::RON,
};
use autoschematic_macros::FieldTypes;
use documented::{Documented, DocumentedFields};
use serde::{Deserialize, Serialize};

use super::addr::GitHubResourceAddress;

#[derive(Debug, Serialize, Deserialize, PartialEq, Documented, DocumentedFields, FieldTypes)]
#[serde(default, deny_unknown_fields)]
/// A GitHub repository with its configuration settings
pub struct GitHubRepository {
    /// A short description of the repository
    pub description: Option<String>,
    /// A URL with more information about the repository
    pub homepage: Option<String>,
    /// An array of topics to help categorize the repository
    pub topics: Vec<String>,
    /// Whether the repository is private. If false, the repository is public
    pub private: bool,
    /// Whether issues are enabled for the repository
    pub has_issues: bool,
    /// Whether projects are enabled for the repository
    pub has_projects: bool,
    /// Whether the wiki is enabled for the repository
    pub has_wiki: bool,
    /// Whether to allow squash merges for pull requests
    pub allow_squash_merge: bool,
    /// Whether to allow merge commits for pull requests
    pub allow_merge_commit: bool,
    /// Whether to allow rebase merges for pull requests
    pub allow_rebase_merge: bool,
    /// Whether to allow auto-merge on pull requests
    pub allow_auto_merge: bool,
    /// Whether to delete head branches when pull requests are merged
    pub delete_branch_on_merge: bool,
    /// The default branch for the repository (e.g., "main" or "master")
    pub default_branch: String,
    /// Whether the repository is archived and read-only
    pub archived: bool,
    /// Whether the repository is disabled
    pub disabled: bool,
    /// Map of collaborators (users or teams) and their permission roles
    pub collaborators: HashMap<CollaboratorPrincipal, Role>,
}

impl Default for GitHubRepository {
    fn default() -> Self {
        Self {
            description: Default::default(),
            homepage: Default::default(),
            topics: Default::default(),
            private: true,
            has_issues: true,
            has_projects: true,
            has_wiki: true,
            allow_squash_merge: true,
            allow_merge_commit: true,
            allow_rebase_merge: true,
            allow_auto_merge: false,
            delete_branch_on_merge: false,
            default_branch: "main".into(),
            archived: false,
            disabled: false,
            collaborators: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Documented, DocumentedFields, FieldTypes)]
#[serde(deny_unknown_fields)]
/// Required status checks that must pass before merging a pull request
pub struct RequiredStatusChecks {
    /// Whether to require branches to be up to date before merging
    pub strict: bool,
    /// The list of status checks that must pass before branches can be merged
    pub contexts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Documented, DocumentedFields, FieldTypes)]
#[serde(deny_unknown_fields)]
/// Pull request review enforcement settings for branch protection
pub struct PullRequestReviewEnforcement {
    /// The number of approving reviews required before a pull request can be merged
    pub required_approving_review_count: u32,
    /// Whether to dismiss approving reviews when new commits are pushed
    pub dismiss_stale_reviews: bool,
    /// Whether to require review from code owners
    pub require_code_owner_reviews: bool,
    /// Whether to require approval of the most recent reviewable push
    pub require_last_push_approval: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Documented, DocumentedFields, FieldTypes)]
#[serde(deny_unknown_fields)]
/// Restrictions on who can push to a protected branch
pub struct BranchRestrictions {
    /// Users allowed to push to the branch
    pub users: Vec<String>,
    /// Teams allowed to push to the branch
    pub teams: Vec<String>,
    /// GitHub Apps allowed to push to the branch
    pub apps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Documented, DocumentedFields, FieldTypes)]
#[serde(deny_unknown_fields)]
/// Branch protection rules that control how a branch can be modified
pub struct BranchProtection {
    /// Status checks that must pass before merging
    pub required_status_checks: Option<RequiredStatusChecks>,
    /// Whether to enforce all configured restrictions for administrators
    pub enforce_admins: bool,
    /// Pull request review requirements
    pub required_pull_request_reviews: Option<PullRequestReviewEnforcement>,
    /// Restrictions on who can push to the branch
    pub restrictions: Option<BranchRestrictions>,
    /// Whether to require a linear commit history (no merge commits)
    pub required_linear_history: bool,
    /// Whether to allow force pushes to the branch
    pub allow_force_pushes: bool,
    /// Whether to allow branch deletions
    pub allow_deletions: bool,
    /// Whether to block creation of matching branches
    pub block_creations: bool,
    /// Whether to require all conversations on code to be resolved before merging
    pub required_conversation_resolution: bool,
    /// Whether to lock the branch, making it read-only
    pub lock_branch: bool,
    /// Whether to allow users with push access to sync from upstream forks
    pub allow_fork_syncing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Documented, DocumentedFields)]
/// A principal that can be granted collaborator access to a repository
pub enum CollaboratorPrincipal {
    /// A GitHub user account by username
    User(String),
    /// A GitHub team by slug/name
    Team(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Documented)]
/// Permission level for repository collaborators
pub enum Role {
    /// Read-only access to the repository
    Read,
    /// Triage access: read plus ability to manage issues and pull requests
    Triage,
    /// Write access: triage plus ability to push to the repository
    Write,
    /// Maintain access: write plus ability to manage the repository without sensitive actions
    Maintain,
    /// Administrator access: full control of the repository
    Admin,
    /// A custom repository role defined in the organization
    Custom(String),
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Read => "read",
            Role::Triage => "triage",
            Role::Write => "write",
            Role::Maintain => "maintain",
            Role::Admin => "admin",
            Role::Custom(s) => s,
        }
        .into()
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "read" => Role::Read,
            "triage" => Role::Triage,
            "write" => Role::Write,
            "maintain" => Role::Maintain,
            "admin" => Role::Admin,
            s => Role::Custom(s.into()),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct CollaboratorSet {
//     pub users: HashMap<String, Role>,
//     #[serde(skip_serializing_if = "HashMap::is_empty")]
//     pub teams: HashMap<String, Role>,
// }

pub enum GitHubResource {
    Repository(GitHubRepository),
    BranchProtection(BranchProtection),
}

impl Resource for GitHubResource {
    fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        let pretty_config = autoschematic_core::util::PrettyConfig::default().struct_names(true);
        match self {
            GitHubResource::Repository(repo) => Ok(RON.to_string_pretty(&repo, pretty_config)?.into()),
            GitHubResource::BranchProtection(protection) => Ok(RON.to_string_pretty(&protection, pretty_config)?.into()),
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
            _ => Err(invalid_addr(&addr)),
        }
    }
}
