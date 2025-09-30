use crate::{
    GitHubConnector,
    addr::GitHubResourceAddress,
    github_ext::{BranchProtectionExt, CollaboratorExt},
    resource,
};
use anyhow::Context;
use autoschematic_core::{
    connector::{GetResourceResponse, Resource, ResourceAddress},
    get_resource_response,
};
use std::path::Path;

impl GitHubConnector {
    pub async fn do_get(&self, addr: &Path) -> anyhow::Result<Option<GetResourceResponse>> {
        let addr = GitHubResourceAddress::from_path(addr)?;

        match addr {
            GitHubResourceAddress::Config => Ok(None),
            GitHubResourceAddress::Repository { owner, repo } => {
                match self.client.read().await.repos(&owner, &repo).get().await {
                    Ok(github_repo) => {
                        let repo_resource = resource::GitHubRepository {
                            description: github_repo.description,
                            homepage: github_repo.homepage,
                            topics: github_repo.topics.unwrap_or_default(),
                            private: github_repo.private.unwrap_or(false),
                            has_issues: github_repo.has_issues.unwrap_or(true),
                            has_projects: github_repo.has_projects.unwrap_or(true),
                            has_wiki: github_repo.has_wiki.unwrap_or(true),
                            allow_squash_merge: github_repo.allow_squash_merge.unwrap_or(true),
                            allow_merge_commit: github_repo.allow_merge_commit.unwrap_or(true),
                            allow_rebase_merge: github_repo.allow_rebase_merge.unwrap_or(true),
                            allow_auto_merge: github_repo.allow_auto_merge.unwrap_or(false),
                            delete_branch_on_merge: github_repo.delete_branch_on_merge.unwrap_or(false),
                            default_branch: github_repo.default_branch.unwrap_or_else(|| "main".to_string()),
                            archived: github_repo.archived.unwrap_or(false),
                            disabled: github_repo.disabled.unwrap_or(false),
                        };

                        get_resource_response!(resource::GitHubResource::Repository(repo_resource))
                    }
                    Err(_) => Ok(None), // Repository doesn't exist
                }
            }
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => {
                match self.client.read().await.get_branch_protection(&owner, &repo, &branch).await {
                    Ok(protection) => {
                        let protection_resource = resource::BranchProtection {
                            required_status_checks: protection.required_status_checks.map(|checks| {
                                resource::RequiredStatusChecks {
                                    strict: checks.strict,
                                    contexts: checks.contexts,
                                }
                            }),
                            enforce_admins: protection.enforce_admins.enabled,
                            required_pull_request_reviews: protection.required_pull_request_reviews.map(|reviews| {
                                resource::PullRequestReviewEnforcement {
                                    required_approving_review_count: reviews.required_approving_review_count.unwrap_or(1),
                                    dismiss_stale_reviews: reviews.dismiss_stale_reviews.unwrap_or(false),
                                    require_code_owner_reviews: reviews.require_code_owner_reviews.unwrap_or(false),
                                    require_last_push_approval: reviews.require_last_push_approval.unwrap_or(false),
                                }
                            }),
                            restrictions: protection.restrictions.map(|restrictions| resource::BranchRestrictions {
                                users: restrictions.users.into_iter().map(|u| u.login).collect(),
                                teams: restrictions.teams.into_iter().map(|t| t.name).collect(),
                                apps: restrictions.apps.into_iter().map(|a| a.name).collect(),
                            }),
                            required_linear_history: protection.required_linear_history.map(|s| s.enabled).unwrap_or(false),
                            allow_force_pushes: protection.allow_force_pushes.map(|s| s.enabled).unwrap_or(false),
                            allow_deletions: protection.allow_deletions.map(|s| s.enabled).unwrap_or(false),
                            block_creations: protection.block_creations.map(|s| s.enabled).unwrap_or(false),
                            required_conversation_resolution: protection
                                .required_conversation_resolution
                                .map(|s| s.enabled)
                                .unwrap_or(false),
                            lock_branch: protection.lock_branch.map(|s| s.enabled).unwrap_or(false),
                            allow_fork_syncing: protection.allow_fork_syncing.map(|s| s.enabled).unwrap_or(true),
                        };

                        get_resource_response!(resource::GitHubResource::BranchProtection(protection_resource))
                    }
                    Err(_) => Ok(None), // Branch protection doesn't exist
                }
            }
            GitHubResourceAddress::Collaborator { owner, repo, username } => {
                match self
                    .client
                    .read()
                    .await
                    .get_collaborator_permission(&owner, &repo, &username)
                    .await
                {
                    Ok(collaborator) => {
                        let collaborator_resource = resource::Collaborator {
                            permissions: resource::CollaboratorPermissions {
                                pull: collaborator.permissions.pull,
                                triage: collaborator.permissions.triage,
                                push: collaborator.permissions.push,
                                maintain: collaborator.permissions.maintain,
                                admin: collaborator.permissions.admin,
                            },
                            role_name: collaborator.role_name,
                        };

                        get_resource_response!(resource::GitHubResource::Collaborator(collaborator_resource))
                    }
                    Err(_) => Ok(None), // Collaborator doesn't exist
                }
            }
        }
    }
}
