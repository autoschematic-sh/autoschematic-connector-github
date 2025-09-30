use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::config::GitHubConnectorConfig;
use crate::resource;
use crate::{addr::GitHubResourceAddress, client::get_client};
use anyhow::bail;
use async_trait::async_trait;
use autoschematic_core::{
    connector::{
        Connector, ConnectorOutbox, FilterResponse, GetResourceResponse, OpExecResponse, PlanResponseElement, Resource,
        ResourceAddress, SkeletonResponse,
    },
    diag::DiagnosticResponse,
    skeleton,
    util::{RON, ron_check_eq, ron_check_syntax},
};
use octocrab::Octocrab;
use tokio::sync::RwLock;

pub mod get;
pub mod list;
pub mod op_exec;
pub mod plan;

#[derive(Default)]
pub struct GitHubConnector {
    prefix: PathBuf,
    client: RwLock<Octocrab>,
    config: RwLock<GitHubConnectorConfig>,
}

#[async_trait]
impl Connector for GitHubConnector {
    async fn new(name: &str, prefix: &Path, outbox: ConnectorOutbox) -> Result<Arc<dyn Connector>, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Arc::new(GitHubConnector {
            prefix: prefix.into(),
            ..Default::default()
        }))
    }

    async fn init(&self) -> anyhow::Result<()> {
        let config: GitHubConnectorConfig = match GitHubConnectorConfig::try_load(&self.prefix)? {
            Some(custom_config) => custom_config,
            None => {
                let client = get_client(None).await?;
                let login = client.current().user().await?.login;

                GitHubConnectorConfig {
                    owners: vec![login],
                    ..Default::default()
                }
            }
        };
        *self.config.write().await = config.clone();
        *self.client.write().await = get_client(Some(config)).await?;

        Ok(())
    }

    async fn filter(&self, addr: &Path) -> Result<FilterResponse, anyhow::Error> {
        if let Ok(_addr) = GitHubResourceAddress::from_path(addr) {
            Ok(FilterResponse::Resource)
        } else {
            Ok(FilterResponse::none())
        }
    }

    async fn list(&self, subpath: &Path) -> Result<Vec<PathBuf>, anyhow::Error> {
        self.do_list(subpath).await
    }

    async fn get(&self, addr: &Path) -> Result<Option<GetResourceResponse>, anyhow::Error> {
        self.do_get(addr).await
    }

    async fn plan(
        &self,
        addr: &Path,
        current: Option<Vec<u8>>,
        desired: Option<Vec<u8>>,
    ) -> Result<Vec<PlanResponseElement>, anyhow::Error> {
        self.do_plan(addr, current, desired).await
    }

    async fn op_exec(&self, addr: &Path, op: &str) -> Result<OpExecResponse, anyhow::Error> {
        self.do_op_exec(addr, op).await
    }

    async fn get_skeletons(&self) -> Result<Vec<SkeletonResponse>, anyhow::Error> {
        let mut res = Vec::new();

        res.push(skeleton!(GitHubResourceAddress::Config, GitHubConnectorConfig::default()));
        // Create example repository skeleton
        res.push(skeleton!(
            GitHubResourceAddress::Repository {
                owner: String::from("[owner]"),
                repo: String::from("[repo_name]"),
            },
            resource::GitHubResource::Repository(resource::GitHubRepository {
                description: Some(String::from("A sample repository")),
                homepage: None,
                topics: vec![String::from("rust"), String::from("autoschematic")],
                private: false,
                has_issues: true,
                has_projects: true,
                has_wiki: true,
                allow_squash_merge: true,
                allow_merge_commit: true,
                allow_rebase_merge: true,
                allow_auto_merge: false,
                delete_branch_on_merge: true,
                default_branch: String::from("main"),
                archived: false,
                disabled: false,
            })
        ));

        // Create example branch protection skeleton
        res.push(skeleton!(
            GitHubResourceAddress::BranchProtection {
                owner: String::from("[owner]"),
                repo: String::from("[repo_name]"),
                branch: String::from("main"),
            },
            resource::GitHubResource::BranchProtection(resource::BranchProtection {
                required_status_checks: Some(resource::RequiredStatusChecks {
                    strict: true,
                    contexts: vec![String::from("ci/tests")],
                }),
                enforce_admins: true,
                required_pull_request_reviews: Some(resource::PullRequestReviewEnforcement {
                    required_approving_review_count: 1,
                    dismiss_stale_reviews: true,
                    require_code_owner_reviews: false,
                    require_last_push_approval: false,
                }),
                restrictions: None,
                required_linear_history: false,
                allow_force_pushes: false,
                allow_deletions: false,
                block_creations: false,
                required_conversation_resolution: true,
                lock_branch: false,
                allow_fork_syncing: true,
            })
        ));

        // Create example collaborator skeleton
        res.push(skeleton!(
            GitHubResourceAddress::Collaborator {
                owner: String::from("[owner]"),
                repo: String::from("[repo_name]"),
                username: String::from("[username]"),
            },
            resource::GitHubResource::Collaborator(resource::Collaborator {
                permissions: resource::CollaboratorPermissions {
                    pull: true,
                    triage: true,
                    push: true,
                    maintain: false,
                    admin: false,
                },
                role_name: String::from("push"),
            })
        ));

        Ok(res)
    }

    async fn eq(&self, addr: &Path, a: &[u8], b: &[u8]) -> anyhow::Result<bool> {
        let addr = GitHubResourceAddress::from_path(addr)?;

        match addr {
            GitHubResourceAddress::Config => Ok(a == b),
            GitHubResourceAddress::Repository { .. } => ron_check_eq::<resource::GitHubRepository>(a, b),
            GitHubResourceAddress::BranchProtection { .. } => ron_check_eq::<resource::BranchProtection>(a, b),
            GitHubResourceAddress::Collaborator { .. } => ron_check_eq::<resource::Collaborator>(a, b),
        }
    }

    async fn diag(&self, addr: &Path, a: &[u8]) -> Result<Option<DiagnosticResponse>, anyhow::Error> {
        let addr = GitHubResourceAddress::from_path(addr)?;

        match addr {
            GitHubResourceAddress::Config => ron_check_syntax::<GitHubConnectorConfig>(a),
            GitHubResourceAddress::Repository { .. } => ron_check_syntax::<resource::GitHubRepository>(a),
            GitHubResourceAddress::BranchProtection { .. } => ron_check_syntax::<resource::BranchProtection>(a),
            GitHubResourceAddress::Collaborator { .. } => ron_check_syntax::<resource::Collaborator>(a),
        }
    }
}
