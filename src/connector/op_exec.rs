use crate::{
    GitHubConnector,
    addr::GitHubResourceAddress,
    github_ext::{
        AddCollaboratorRequest, BranchProtectionOpsExt, CollaboratorOpsExt, CreateBranchProtectionRequest,
        CreateRepositoryRequest, RepositoryOpsExt, UpdateRepositoryRequest,
    },
    op::GitHubConnectorOp,
};
use anyhow::bail;
use autoschematic_core::{
    connector::{ConnectorOp, OpExecResponse, ResourceAddress},
    error_util::invalid_op,
};
use std::path::Path;

impl GitHubConnector {
    pub async fn do_op_exec(&self, addr: &Path, op: &str) -> anyhow::Result<OpExecResponse> {
        let addr = GitHubResourceAddress::from_path(addr)?;
        let op = GitHubConnectorOp::from_str(op)?;

        match &addr {
            GitHubResourceAddress::Config => Err(invalid_op(&addr, &op)),
            GitHubResourceAddress::Repository { owner, repo } => {
                let client = self.client.read().await.clone();

                match op {
                    GitHubConnectorOp::CreateRepository(repo_config) => {
                        let create_request = CreateRepositoryRequest {
                            name: repo.clone(),
                            description: repo_config.description.clone(),
                            homepage: repo_config.homepage.clone(),
                            private: repo_config.private,
                            has_issues: repo_config.has_issues,
                            has_projects: repo_config.has_projects,
                            has_wiki: repo_config.has_wiki,
                            allow_squash_merge: repo_config.allow_squash_merge,
                            allow_merge_commit: repo_config.allow_merge_commit,
                            allow_rebase_merge: repo_config.allow_rebase_merge,
                            allow_auto_merge: repo_config.allow_auto_merge,
                            delete_branch_on_merge: repo_config.delete_branch_on_merge,
                            default_branch: Some(repo_config.default_branch.clone()),
                        };

                        match client.create_repository(owner, &create_request).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!("Created GitHub repository {}/{}", owner, repo)),
                            }),
                            Err(e) => bail!("Failed to create repository {}/{}: {}", owner, repo, e),
                        }
                    }
                    GitHubConnectorOp::UpdateRepository(_old_config, new_config) => {
                        let update_request = UpdateRepositoryRequest {
                            name: None, // Can't rename via this API
                            description: new_config.description.clone(),
                            homepage: new_config.homepage.clone(),
                            private: Some(new_config.private),
                            has_issues: Some(new_config.has_issues),
                            has_projects: Some(new_config.has_projects),
                            has_wiki: Some(new_config.has_wiki),
                            allow_squash_merge: Some(new_config.allow_squash_merge),
                            allow_merge_commit: Some(new_config.allow_merge_commit),
                            allow_rebase_merge: Some(new_config.allow_rebase_merge),
                            allow_auto_merge: Some(new_config.allow_auto_merge),
                            delete_branch_on_merge: Some(new_config.delete_branch_on_merge),
                            default_branch: Some(new_config.default_branch.clone()),
                            archived: Some(new_config.archived),
                        };

                        match client.update_repository(owner, repo, &update_request).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!("Updated GitHub repository {}/{}", owner, repo)),
                            }),
                            Err(e) => bail!("Failed to update repository {}/{}: {:#?}", owner, repo, e),
                        }
                    }
                    GitHubConnectorOp::DeleteRepository => match client.delete_repository(owner, repo).await {
                        Ok(_) => Ok(OpExecResponse {
                            outputs: None,
                            friendly_message: Some(format!("Deleted GitHub repository {}/{}", owner, repo)),
                        }),
                        Err(e) => bail!("Failed to delete repository {}/{}: {:#?}", owner, repo, e),
                    },
                    _ => Err(invalid_op(&addr, &op)),
                }
            }
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => {
                let client = self.client.read().await.clone();

                match op {
                    GitHubConnectorOp::CreateBranchProtection(protection_config) => {
                        let create_request = CreateBranchProtectionRequest {
                            required_status_checks: protection_config.required_status_checks.as_ref().map(|checks| {
                                crate::github_ext::GitHubRequiredStatusChecks {
                                    strict: checks.strict,
                                    contexts: checks.contexts.clone(),
                                }
                            }),
                            enforce_admins: protection_config.enforce_admins,
                            required_pull_request_reviews: protection_config.required_pull_request_reviews.as_ref().map(
                                |reviews| crate::github_ext::GitHubPullRequestReviewEnforcement {
                                    required_approving_review_count: Some(reviews.required_approving_review_count),
                                    dismiss_stale_reviews: Some(reviews.dismiss_stale_reviews),
                                    require_code_owner_reviews: Some(reviews.require_code_owner_reviews),
                                    require_last_push_approval: Some(reviews.require_last_push_approval),
                                },
                            ),
                            restrictions: protection_config.restrictions.as_ref().map(|restrictions| {
                                crate::github_ext::GitHubBranchRestrictions {
                                    users: restrictions
                                        .users
                                        .iter()
                                        .map(|u| crate::github_ext::GitHubUser { login: u.clone() })
                                        .collect(),
                                    teams: restrictions
                                        .teams
                                        .iter()
                                        .map(|t| crate::github_ext::GitHubTeam { name: t.clone() })
                                        .collect(),
                                    apps: restrictions
                                        .apps
                                        .iter()
                                        .map(|a| crate::github_ext::GitHubApp { name: a.clone() })
                                        .collect(),
                                }
                            }),
                            required_linear_history: Some(protection_config.required_linear_history),
                            allow_force_pushes: Some(protection_config.allow_force_pushes),
                            allow_deletions: Some(protection_config.allow_deletions),
                            block_creations: Some(protection_config.block_creations),
                            required_conversation_resolution: Some(protection_config.required_conversation_resolution),
                            lock_branch: Some(protection_config.lock_branch),
                            allow_fork_syncing: Some(protection_config.allow_fork_syncing),
                        };

                        match client.create_branch_protection(owner, repo, branch, &create_request).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!(
                                    "Created branch protection for {}/{} branch {}",
                                    owner, repo, branch
                                )),
                            }),
                            Err(e) => bail!(
                                "Failed to create branch protection for {}/{} branch {}: {:#?}",
                                owner,
                                repo,
                                branch,
                                e
                            ),
                        }
                    }
                    GitHubConnectorOp::UpdateBranchProtection(_old_config, new_config) => {
                        let update_request = CreateBranchProtectionRequest {
                            required_status_checks: new_config.required_status_checks.as_ref().map(|checks| {
                                crate::github_ext::GitHubRequiredStatusChecks {
                                    strict: checks.strict,
                                    contexts: checks.contexts.clone(),
                                }
                            }),
                            enforce_admins: new_config.enforce_admins,
                            required_pull_request_reviews: new_config.required_pull_request_reviews.as_ref().map(|reviews| {
                                crate::github_ext::GitHubPullRequestReviewEnforcement {
                                    required_approving_review_count: Some(reviews.required_approving_review_count),
                                    dismiss_stale_reviews: Some(reviews.dismiss_stale_reviews),
                                    require_code_owner_reviews: Some(reviews.require_code_owner_reviews),
                                    require_last_push_approval: Some(reviews.require_last_push_approval),
                                }
                            }),
                            restrictions: new_config.restrictions.as_ref().map(|restrictions| {
                                crate::github_ext::GitHubBranchRestrictions {
                                    users: restrictions
                                        .users
                                        .iter()
                                        .map(|u| crate::github_ext::GitHubUser { login: u.clone() })
                                        .collect(),
                                    teams: restrictions
                                        .teams
                                        .iter()
                                        .map(|t| crate::github_ext::GitHubTeam { name: t.clone() })
                                        .collect(),
                                    apps: restrictions
                                        .apps
                                        .iter()
                                        .map(|a| crate::github_ext::GitHubApp { name: a.clone() })
                                        .collect(),
                                }
                            }),
                            required_linear_history: Some(new_config.required_linear_history),
                            allow_force_pushes: Some(new_config.allow_force_pushes),
                            allow_deletions: Some(new_config.allow_deletions),
                            block_creations: Some(new_config.block_creations),
                            required_conversation_resolution: Some(new_config.required_conversation_resolution),
                            lock_branch: Some(new_config.lock_branch),
                            allow_fork_syncing: Some(new_config.allow_fork_syncing),
                        };

                        match client.update_branch_protection(owner, repo, branch, &update_request).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!(
                                    "Updated branch protection for {}/{} branch {}",
                                    owner, repo, branch
                                )),
                            }),
                            Err(e) => bail!(
                                "Failed to update branch protection for {}/{} branch {}: {:#?}",
                                owner,
                                repo,
                                branch,
                                e
                            ),
                        }
                    }
                    GitHubConnectorOp::DeleteBranchProtection => {
                        match client.delete_branch_protection(owner, repo, branch).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!(
                                    "Removed branch protection for {}/{} branch {}",
                                    owner, repo, branch
                                )),
                            }),
                            Err(e) => bail!(
                                "Failed to remove branch protection for {}/{} branch {}: {:#?}",
                                owner,
                                repo,
                                branch,
                                e
                            ),
                        }
                    }
                    _ => Err(invalid_op(&addr, &op)),
                }
            }
            GitHubResourceAddress::Collaborator { owner, repo, username } => {
                let client = self.client.read().await.clone();

                match op {
                    GitHubConnectorOp::AddCollaborator(collaborator_config) => {
                        let add_request = AddCollaboratorRequest {
                            permission: collaborator_config.role_name.clone(),
                        };

                        match client.add_collaborator(owner, repo, username, &add_request).await {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!("Added collaborator {} to {}/{}", username, owner, repo)),
                            }),
                            Err(e) => bail!("Failed to add collaborator {} to {}/{}: {}", username, owner, repo, e),
                        }
                    }
                    GitHubConnectorOp::UpdateCollaboratorPermission(_old_config, new_config) => {
                        let update_request = AddCollaboratorRequest {
                            permission: new_config.role_name.clone(),
                        };

                        match client
                            .update_collaborator_permission(owner, repo, username, &update_request)
                            .await
                        {
                            Ok(_) => Ok(OpExecResponse {
                                outputs: None,
                                friendly_message: Some(format!(
                                    "Updated collaborator {} permissions for {}/{}",
                                    username, owner, repo
                                )),
                            }),
                            Err(e) => bail!(
                                "Failed to update collaborator {} permissions for {}/{}: {:#?}",
                                username,
                                owner,
                                repo,
                                e
                            ),
                        }
                    }
                    GitHubConnectorOp::RemoveCollaborator => match client.remove_collaborator(owner, repo, username).await {
                        Ok(_) => Ok(OpExecResponse {
                            outputs: None,
                            friendly_message: Some(format!("Removed collaborator {} from {}/{}", username, owner, repo)),
                        }),
                        Err(e) => bail!("Failed to remove collaborator {} from {}/{}: {:#?}", username, owner, repo, e),
                    },
                    _ => Err(invalid_op(&addr, &op)),
                }
            }
        }
    }
}
