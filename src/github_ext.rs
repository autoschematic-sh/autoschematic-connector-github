use std::collections::HashMap;

use async_trait::async_trait;
use octocrab::{Octocrab, Page, Result};
use serde::{Deserialize, Serialize};

use crate::resource::{CollaboratorPrincipal, Role};

// GitHub API response structures for branch protection
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubBranchProtection {
    pub required_status_checks: Option<GitHubRequiredStatusChecks>,
    pub enforce_admins: GitHubEnforceAdmins,
    pub required_pull_request_reviews: Option<GitHubPullRequestReviewEnforcement>,
    pub restrictions: Option<GitHubBranchRestrictions>,
    pub required_linear_history: Option<GitHubBooleanSetting>,
    pub allow_force_pushes: Option<GitHubBooleanSetting>,
    pub allow_deletions: Option<GitHubBooleanSetting>,
    pub block_creations: Option<GitHubBooleanSetting>,
    pub required_conversation_resolution: Option<GitHubBooleanSetting>,
    pub lock_branch: Option<GitHubBooleanSetting>,
    pub allow_fork_syncing: Option<GitHubBooleanSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRequiredStatusChecks {
    pub strict: bool,
    pub contexts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEnforceAdmins {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPullRequestReviewEnforcement {
    pub required_approving_review_count: Option<u32>,
    pub dismiss_stale_reviews: Option<bool>,
    pub require_code_owner_reviews: Option<bool>,
    pub require_last_push_approval: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubBranchRestrictions {
    pub users: Vec<GitHubUser>,
    pub teams: Vec<GitHubTeam>,
    pub apps: Vec<GitHubApp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTeam {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubApp {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubBooleanSetting {
    pub enabled: bool,
}

// Collaborator response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubCollaboratorPermissions {
    pub pull: bool,
    pub triage: bool,
    pub push: bool,
    pub maintain: bool,
    pub admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubCollaborator {
    pub permissions: GitHubCollaboratorPermissions,
    pub role_name: String,
}

#[async_trait]
pub trait BranchProtectionExt {
    async fn get_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<GitHubBranchProtection>;
}

#[async_trait]
impl BranchProtectionExt for Octocrab {
    async fn get_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<GitHubBranchProtection> {
        let route = format!("/repos/{}/{}/branches/{}/protection", owner, repo, branch);
        self.get(route, None::<&()>).await
    }
}

#[async_trait]
pub trait CollaboratorExt {
    async fn get_collaborator_permission(&self, owner: &str, repo: &str, username: &str) -> Result<GitHubCollaborator>;
}

#[async_trait]
impl CollaboratorExt for Octocrab {
    async fn get_collaborator_permission(&self, owner: &str, repo: &str, username: &str) -> Result<GitHubCollaborator> {
        let route = format!("/repos/{}/{}/collaborators/{}/permission", owner, repo, username);
        self.get(route, None::<&()>).await
    }
}

// Additional structures for listing
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubBranch {
    pub name: String,
    pub protected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubCollaboratorInfo {
    pub login: String,
    pub role_name: String,
}

#[async_trait]
pub trait ListExt {
    // async fn list_user_repos(&self, username: &str) -> Result<octocrab::Page<octocrab::models::Repository>>;
    async fn list_repo_branches(&self, owner: &str, repo: &str) -> Result<octocrab::Page<GitHubBranch>>;
    async fn list_repo_collaborators(
        &self,
        owner: &str,
        repo: &str,
        affiliation: Option<&str>,
    ) -> Result<HashMap<CollaboratorPrincipal, Role>>;
}

#[async_trait]
impl ListExt for Octocrab {
    // async fn list_user_repos(&self, username: &str) -> Result<octocrab::Page<octocrab::models::Repository>> {
    //     let route = format!("/users/{}/repos", username);
    //     self.get(route, None::<&()>).await
    // }

    async fn list_repo_branches(&self, owner: &str, repo: &str) -> Result<octocrab::Page<GitHubBranch>> {
        let route = format!("/repos/{}/{}/branches", owner, repo);
        self.get(route, None::<&()>).await
    }

    async fn list_repo_collaborators(
        &self,
        owner: &str,
        repo: &str,
        affiliation: Option<&str>,
    ) -> Result<HashMap<CollaboratorPrincipal, Role>> {
        let mut res = HashMap::new();

        #[derive(serde::Serialize)]
        struct CollabQuery<'a> {
            affiliation: &'a str, // "direct" | "outside" | "all"
            per_page: u8,
            page: u32,
        }

        let route = format!("/repos/{}/{}/collaborators", owner, repo);
        let users: Page<GitHubCollaboratorInfo> = self
            .get(
                route,
                affiliation
                    .map(|affiliation| CollabQuery {
                        affiliation: affiliation,
                        per_page: 100,
                        page: 1,
                    })
                    .as_ref(),
            )
            .await?;

        let users = self.all_pages(users).await?;

        for user in users {
            res.insert(CollaboratorPrincipal::User(user.login), Role::from_str(&user.role_name));
        }

        Ok(res)
    }

    // async fn list_repo_collaborators(
    //     &self,
    //     owner: &str,
    //     repo: &str,
    //     affiliation: Option<&str>,
    // ) -> Result<octocrab::Page<GitHubCollaboratorInfo>> {
    //     #[derive(serde::Serialize)]
    //     struct CollabQuery<'a> {
    //         affiliation: &'a str, // "direct" | "outside" | "all"
    //         per_page: u8,
    //         page: u32,
    //     }

    //     let route = format!("/repos/{}/{}/collaborators", owner, repo);
    //     self.get(
    //         route,
    //         affiliation
    //             .map(|affiliation| CollabQuery {
    //                 affiliation: affiliation,
    //                 per_page: 100,
    //                 page: 1,
    //             })
    //             .as_ref(),
    //     )
    //     .await
    // }
}

// Structures for repository operations
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub private: bool,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_wiki: bool,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase_merge: bool,
    pub allow_auto_merge: bool,
    pub delete_branch_on_merge: bool,
    pub default_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub private: Option<bool>,
    pub has_issues: Option<bool>,
    pub has_projects: Option<bool>,
    pub has_wiki: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub allow_auto_merge: Option<bool>,
    pub delete_branch_on_merge: Option<bool>,
    pub default_branch: Option<String>,
    pub archived: Option<bool>,
}

// Structures for branch protection operations
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBranchProtectionRequest {
    pub required_status_checks: Option<GitHubRequiredStatusChecks>,
    pub enforce_admins: bool,
    pub required_pull_request_reviews: Option<GitHubPullRequestReviewEnforcement>,
    pub restrictions: Option<GitHubBranchRestrictions>,
    pub required_linear_history: Option<bool>,
    pub allow_force_pushes: Option<bool>,
    pub allow_deletions: Option<bool>,
    pub block_creations: Option<bool>,
    pub required_conversation_resolution: Option<bool>,
    pub lock_branch: Option<bool>,
    pub allow_fork_syncing: Option<bool>,
}

// Structures for collaborator operations
#[derive(Debug, Serialize, Deserialize)]
pub struct AddCollaboratorRequest {
    pub permission: String, // "pull", "triage", "push", "maintain", "admin"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTeamCollaboratorRequest {
    pub permission: String, // "pull", "triage", "push", "maintain", "admin"
}

#[async_trait]
pub trait RepositoryOpsExt {
    async fn create_repository(&self, owner: &str, repo_data: &CreateRepositoryRequest)
    -> Result<octocrab::models::Repository>;
    async fn update_repository(
        &self,
        owner: &str,
        repo: &str,
        repo_data: &UpdateRepositoryRequest,
    ) -> Result<octocrab::models::Repository>;
    async fn delete_repository(&self, owner: &str, repo: &str) -> Result<()>;
}

#[async_trait]
impl RepositoryOpsExt for Octocrab {
    async fn create_repository(
        &self,
        _owner: &str,
        repo_data: &CreateRepositoryRequest,
    ) -> Result<octocrab::models::Repository> {
        let route = format!("/user/repos");
        self.post(route, Some(repo_data)).await
    }

    async fn update_repository(
        &self,
        owner: &str,
        repo: &str,
        repo_data: &UpdateRepositoryRequest,
    ) -> Result<octocrab::models::Repository> {
        let route = format!("/repos/{}/{}", owner, repo);
        self.patch(route, Some(repo_data)).await
    }

    async fn delete_repository(&self, owner: &str, repo: &str) -> Result<()> {
        let route = format!("/repos/{}/{}", owner, repo);
        self.delete(route, None::<&()>).await
    }
}

#[async_trait]
pub trait BranchProtectionOpsExt {
    async fn create_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        protection_data: &CreateBranchProtectionRequest,
    ) -> Result<GitHubBranchProtection>;
    async fn update_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        protection_data: &CreateBranchProtectionRequest,
    ) -> Result<GitHubBranchProtection>;
    async fn delete_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<()>;
}

#[async_trait]
impl BranchProtectionOpsExt for Octocrab {
    async fn create_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        protection_data: &CreateBranchProtectionRequest,
    ) -> Result<GitHubBranchProtection> {
        let route = format!("/repos/{}/{}/branches/{}/protection", owner, repo, branch);
        self.put(route, Some(protection_data)).await
    }

    async fn update_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        protection_data: &CreateBranchProtectionRequest,
    ) -> Result<GitHubBranchProtection> {
        let route = format!("/repos/{}/{}/branches/{}/protection", owner, repo, branch);
        self.put(route, Some(protection_data)).await
    }

    async fn delete_branch_protection(&self, owner: &str, repo: &str, branch: &str) -> Result<()> {
        let route = format!("/repos/{}/{}/branches/{}/protection", owner, repo, branch);
        self.delete(route, None::<&()>).await
    }
}

#[async_trait]
pub trait CollaboratorOpsExt {
    async fn add_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission_data: &AddCollaboratorRequest,
    ) -> Result<()>;
    async fn update_collaborator_permission(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission_data: &AddCollaboratorRequest,
    ) -> Result<()>;
    async fn remove_collaborator(&self, owner: &str, repo: &str, username: &str) -> Result<()>;
    async fn add_team_to_repository(
        &self,
        owner: &str,
        repo: &str,
        team_slug: &str,
        permission_data: &AddTeamCollaboratorRequest,
    ) -> Result<()>;
    async fn update_team_permission(
        &self,
        owner: &str,
        repo: &str,
        team_slug: &str,
        permission_data: &AddTeamCollaboratorRequest,
    ) -> Result<()>;
    async fn remove_team_from_repository(&self, owner: &str, repo: &str, team_slug: &str) -> Result<()>;
}

#[async_trait]
impl CollaboratorOpsExt for Octocrab {
    async fn add_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission_data: &AddCollaboratorRequest,
    ) -> Result<()> {
        let route = format!("/repos/{}/{}/collaborators/{}", owner, repo, username);
        self.put(route, Some(permission_data)).await
    }

    async fn update_collaborator_permission(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission_data: &AddCollaboratorRequest,
    ) -> Result<()> {
        let route = format!("/repos/{}/{}/collaborators/{}", owner, repo, username);
        self.put(route, Some(permission_data)).await
    }

    async fn remove_collaborator(&self, owner: &str, repo: &str, username: &str) -> Result<()> {
        let route = format!("/repos/{}/{}/collaborators/{}", owner, repo, username);
        self.delete(route, None::<&()>).await
    }

    async fn add_team_to_repository(
        &self,
        owner: &str,
        repo: &str,
        team_slug: &str,
        permission_data: &AddTeamCollaboratorRequest,
    ) -> Result<()> {
        let route = format!("/orgs/{}/teams/{}/repos/{}/{}", owner, team_slug, owner, repo);
        self.put(route, Some(permission_data)).await
    }

    async fn update_team_permission(
        &self,
        owner: &str,
        repo: &str,
        team_slug: &str,
        permission_data: &AddTeamCollaboratorRequest,
    ) -> Result<()> {
        let route = format!("/orgs/{}/teams/{}/repos/{}/{}", owner, team_slug, owner, repo);
        self.put(route, Some(permission_data)).await
    }

    async fn remove_team_from_repository(&self, owner: &str, repo: &str, team_slug: &str) -> Result<()> {
        let route = format!("/orgs/{}/teams/{}/repos/{}/{}", owner, team_slug, owner, repo);
        self.delete(route, None::<&()>).await
    }
}
