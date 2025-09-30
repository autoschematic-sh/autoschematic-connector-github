use autoschematic_core::connector::ResourceAddress;
use autoschematic_core::glob::addr_matches_filter;
use futures_util::TryStreamExt;
use tokio::pin;

use crate::{GitHubConnector, addr::GitHubResourceAddress, github_ext::ListExt};
use std::path::{Path, PathBuf};

impl GitHubConnector {
    pub async fn do_list(&self, subpath: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut results = Vec::new();

        let owners = self.config.read().await.owners.clone();

        for owner in owners {
            if !addr_matches_filter(&PathBuf::from(format!("github/{owner}/")), subpath) {
                continue;
            }

            let client = self.client.read().await.clone();

            match client.list_user_repos(&owner).await {
                Ok(repos_page) => {
                    let repo_stream = repos_page.into_stream(&client);
                    pin!(repo_stream);

                    while let Some(repo) = repo_stream.try_next().await? {
                        let addr = GitHubResourceAddress::Repository {
                            owner: owner.clone(),
                            repo: repo.name.clone(),
                        };
                        results.push(addr.to_path_buf());

                        match self.client.read().await.list_repo_branches(&owner, &repo.name).await {
                            Ok(branch_page) => {
                                let branch_stream = branch_page.into_stream(&client);
                                pin!(branch_stream);

                                while let Some(branch) = branch_stream.try_next().await? {
                                    if branch.protected {
                                        let addr = GitHubResourceAddress::BranchProtection {
                                            owner: owner.clone(),
                                            repo: repo.name.clone(),
                                            branch: branch.name,
                                        };
                                        results.push(addr.to_path_buf());
                                    }
                                }
                            }
                            Err(_) => {}
                        }

                        match self.client.read().await.list_repo_collaborators(&owner, &repo.name).await {
                            Ok(collaborator_page) => {
                                let collaborator_stream = collaborator_page.into_stream(&client);
                                pin!(collaborator_stream);

                                while let Some(collaborator) = collaborator_stream.try_next().await? {
                                    let addr = GitHubResourceAddress::Collaborator {
                                        owner: owner.clone(),
                                        repo: repo.name.clone(),
                                        username: collaborator.login.clone(),
                                    };
                                    results.push(addr.to_path_buf());
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{:#?}", e);
                }
            }
        }

        Ok(results)
    }
}
