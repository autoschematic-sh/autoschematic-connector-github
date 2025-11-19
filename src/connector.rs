use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::resource::{self, BranchProtection, GitHubRepository};
use crate::{addr::GitHubResourceAddress, client::get_client};
use crate::{
    config::GitHubConnectorConfig,
    resource::{CollaboratorPrincipal, Role},
};
use async_trait::async_trait;
use autoschematic_core::{
    connector::{
        Connector, ConnectorOutbox, DocIdent, FilterResponse, GetDocResponse, GetResourceResponse, OpExecResponse,
        PlanResponseElement, Resource, ResourceAddress, SkeletonResponse,
    },
    diag::DiagnosticResponse,
    doc_dispatch, skeleton,
    util::{ron_check_eq, ron_check_syntax},
};
use octocrab::Octocrab;
use tokio::sync::{RwLock, Semaphore};

pub mod get;
pub mod list;
pub mod op_exec;
pub mod plan;

// #[derive(Default)]
pub struct GitHubConnector {
    prefix: PathBuf,
    client: RwLock<Octocrab>,
    config: RwLock<GitHubConnectorConfig>,
    semaphore: RwLock<tokio::sync::Semaphore>,
}

impl Default for GitHubConnector {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            client: Default::default(),
            config: Default::default(),
            semaphore: RwLock::new(tokio::sync::Semaphore::const_new(1)),
        }
    }
}

#[async_trait]
impl Connector for GitHubConnector {
    async fn new(_name: &str, prefix: &Path, _outbox: ConnectorOutbox) -> Result<Arc<dyn Connector>, anyhow::Error>
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
                    users: vec![login],
                    ..Default::default()
                }
            }
        };

        *self.config.write().await = config.clone();
        *self.semaphore.write().await = Semaphore::new(config.concurrent_requests);
        *self.client.write().await = get_client(Some(config)).await?;

        Ok(())
    }

    async fn filter(&self, addr: &Path) -> Result<FilterResponse, anyhow::Error> {
        if let Ok(addr) = GitHubResourceAddress::from_path(addr) {
            match addr {
                GitHubResourceAddress::Config => Ok(FilterResponse::Config),
                _ => Ok(FilterResponse::Resource),
            }
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

        let mut collaborators = HashMap::new();
        collaborators.insert(CollaboratorPrincipal::User("alice".into()), Role::Admin);
        collaborators.insert(CollaboratorPrincipal::User("bob".into()), Role::Write);
        collaborators.insert(CollaboratorPrincipal::Team("core-team".into()), Role::Maintain);

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
                collaborators: collaborators
            })
        ));

        res.push(skeleton!(
            GitHubResourceAddress::BranchProtection {
                owner: String::from("[owner]"),
                repo: String::from("[repo_name]"),
                branch: String::from("[branch_name]"),
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

        Ok(res)
    }

    async fn eq(&self, addr: &Path, a: &[u8], b: &[u8]) -> anyhow::Result<bool> {
        let addr = GitHubResourceAddress::from_path(addr)?;

        match addr {
            GitHubResourceAddress::Config => ron_check_eq::<GitHubConnectorConfig>(a, b),
            GitHubResourceAddress::Repository { .. } => ron_check_eq::<resource::GitHubRepository>(a, b),
            GitHubResourceAddress::BranchProtection { .. } => ron_check_eq::<resource::BranchProtection>(a, b),
        }
    }

    async fn diag(&self, addr: &Path, a: &[u8]) -> Result<Option<DiagnosticResponse>, anyhow::Error> {
        let addr = GitHubResourceAddress::from_path(addr)?;

        match addr {
            GitHubResourceAddress::Config => ron_check_syntax::<GitHubConnectorConfig>(a),
            GitHubResourceAddress::Repository { .. } => ron_check_syntax::<resource::GitHubRepository>(a),
            GitHubResourceAddress::BranchProtection { .. } => ron_check_syntax::<resource::BranchProtection>(a),
        }
    }

    async fn get_docstring(&self, _addr: &Path, ident: DocIdent) -> Result<Option<GetDocResponse>, anyhow::Error> {
        doc_dispatch!(
            ident,
            [GitHubConnectorConfig, GitHubRepository, BranchProtection,],
            [CollaboratorPrincipal::User(String::new())]
        )
        // match ident {
        //     DocIdent::Struct { name } => match name.as_str() {
        //         "GitHubConnectorConfig" => Ok(Some(GetDocResponse::from_documented::<GitHubConnectorConfig>())),
        //         "GitHubRepository" => Ok(Some(GetDocResponse::from_documented::<GitHubRepository>())),
        //         "BranchProtection" => Ok(Some(GetDocResponse::from_documented::<BranchProtection>())),
        //         "CollaboratorPrincipal" => Ok(Some(GetDocResponse::from_documented::<CollaboratorPrincipal>())),
        //         _ => Ok(None),
        //     },
        //     DocIdent::Field { parent, name } => match parent.as_str() {
        //         "GitHubConnectorConfig" => Ok(Some(GitHubConnectorConfig::get_field_docs(name)?.into())),
        //         "GitHubRepository" => Ok(Some(GitHubRepository::get_field_docs(name)?.into())),
        //         "BranchProtection" => Ok(Some(BranchProtection::get_field_docs(name)?.into())),
        //         "CollaboratorPrincipal" => Ok(Some(CollaboratorPrincipal::get_field_docs(name)?.into())),
        //         _ => Ok(None),
        //     },
        // }
    }
}
