use autoschematic_core::connector::ResourceAddress;
use autoschematic_core::glob::addr_matches_filter;
use futures_util::TryStreamExt;
use octocrab::{Octocrab, Page, models::Repository};
use tokio::pin;

use crate::{GitHubConnector, addr::GitHubResourceAddress, github_ext::ListExt};
use std::path::{Path, PathBuf};

pub async fn list_repo_stream(owner: String, client: &Octocrab, page: Page<Repository>) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    let repo_stream = page.into_stream(&client);
    pin!(repo_stream);

    while let Some(repo) = repo_stream.try_next().await? {
        let addr = GitHubResourceAddress::Repository {
            owner: owner.clone(),
            repo: repo.name.clone(),
        };
        results.push(addr.to_path_buf());

        match client.list_repo_branches(&owner, &repo.name).await {
            Ok(branch_page) => {
                let branch_stream = branch_page.into_stream(&client);
                pin!(branch_stream);

                while let Some(branch) = branch_stream.try_next().await? {
                    tracing::info!("...{}...", branch.name);
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

        // match client.list_repo_collaborators(&owner, &repo.name).await {
        //     Ok(collaborator_page) => {
        //         let collaborator_stream = collaborator_page.into_stream(&client);
        //         pin!(collaborator_stream);

        //         while let Some(collaborator) = collaborator_stream.try_next().await? {
        //             if collaborator.login == owner {
        //                 continue;
        //             }

        //             let addr = GitHubResourceAddress::CollaboratorSet {
        //                 owner: owner.clone(),
        //                 repo: repo.name.clone(),
        //             };
        //             results.push(addr.to_path_buf());
        //         }
        //     }
        //     Err(_) => {}
        // }
    }
    Ok(results)
}

impl GitHubConnector {
    pub async fn do_list(&self, subpath: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut results = Vec::new();
        let users = self.config.read().await.users.clone();

        for user in users {
            if !addr_matches_filter(&PathBuf::from(format!("github/{user}/")), subpath) {
                continue;
            }

            let client = self.client.read().await.clone();

            let page = client
                .current()
                .list_repos_for_authenticated_user()
                .visibility("all")
                .affiliation("owner")
                .per_page(100)
                .send()
                .await;

            match page {
                Ok(repos_page) => {
                    results.append(&mut list_repo_stream(user, &client, repos_page).await?);
                }
                Err(e) => {
                    tracing::error!("{:#?}", e);
                }
            }
        }

        let orgs = self.config.read().await.orgs.clone();

        for org in orgs {
            if !addr_matches_filter(&PathBuf::from(format!("github/{org}/")), subpath) {
                continue;
            }

            let client = self.client.read().await.clone();

            match client.orgs(&org).list_repos().send().await {
                Ok(repos_page) => {
                    results.append(&mut list_repo_stream(org, &client, repos_page).await?);
                }
                Err(e) => {
                    tracing::error!("{:#?}", e);
                }
            }
        }

        Ok(results)
    }
}
