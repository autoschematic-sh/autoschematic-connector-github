use crate::{GitHubConnector, addr::GitHubResourceAddress, op::GitHubConnectorOp, resource};
use autoschematic_core::{
    connector::{ConnectorOp, PlanResponseElement, ResourceAddress},
    connector_op,
    util::{RON, diff_ron_values},
};
use std::{collections::HashMap, path::Path};

impl GitHubConnector {
    pub async fn do_plan(
        &self,
        addr: &Path,
        current: Option<Vec<u8>>,
        desired: Option<Vec<u8>>,
    ) -> anyhow::Result<Vec<PlanResponseElement>> {
        let current = current.map(String::from_utf8);
        let desired = desired.map(String::from_utf8);

        let addr = GitHubResourceAddress::from_path(addr)?;
        let mut res = Vec::new();

        match addr {
            GitHubResourceAddress::Config => {}
            GitHubResourceAddress::Repository { owner, repo } => match (current, desired) {
                (None, None) => {}
                (None, Some(desired)) => {
                    let new_repo: resource::GitHubRepository = RON.from_str(&desired?)?;

                    res.push(connector_op!(
                        GitHubConnectorOp::CreateRepository(new_repo),
                        format!("Create GitHub repository {}/{}", owner, repo)
                    ));
                }
                (Some(_), None) => {
                    res.push(connector_op!(
                        GitHubConnectorOp::DeleteRepository,
                        format!("Delete GitHub repository {}/{}", owner, repo)
                    ));
                }
                (Some(current), Some(desired)) => {
                    if current != desired {
                        let mut old_repo: resource::GitHubRepository = RON.from_str(&current?)?;
                        let mut new_repo: resource::GitHubRepository = RON.from_str(&desired?)?;

                        if old_repo.collaborators != new_repo.collaborators {
                            for (k, v) in &new_repo.collaborators {
                                if !old_repo.collaborators.contains_key(k) {
                                    res.push(connector_op!(
                                        GitHubConnectorOp::AddCollaborator(k.clone(), v.clone()),
                                        format!("Add Collaborator {:?} to repo {}/{} with role {:?}", k, owner, repo, v)
                                    ));
                                } else if old_repo.collaborators.get(k) != Some(v) {
                                    res.push(connector_op!(
                                        GitHubConnectorOp::UpdateCollaborator(k.clone(), v.clone()),
                                        format!("Update Collaborator {:?} on repo {}/{} to role {:?}", k, owner, repo, v)
                                    ));
                                }
                            }
                            for (k, _) in &old_repo.collaborators {
                                if !new_repo.collaborators.contains_key(k) {
                                    res.push(connector_op!(
                                        GitHubConnectorOp::RemoveCollaborator(k.clone()),
                                        format!("Remove Collaborator {:?} from repo {}/{}", k, owner, repo)
                                    ));
                                }
                            }
                        }
                        
                        // Now that we've computed the collaborator updates manually, exclude them from the diff.
                        old_repo.collaborators = HashMap::new();
                        new_repo.collaborators = HashMap::new();

                        // Only update repository if other fields changed
                        if old_repo != new_repo {
                            let diff = diff_ron_values(&old_repo, &new_repo).unwrap_or_default();
                            res.push(connector_op!(
                                GitHubConnectorOp::UpdateRepository(new_repo),
                                format!("Update GitHub repository {}/{}\n{}", owner, repo, diff)
                            ));
                        }
                    }
                }
            },
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => match (current, desired) {
                (None, None) => {}
                (None, Some(desired)) => {
                    let new_protection: resource::BranchProtection = RON.from_str(&desired?)?;

                    res.push(connector_op!(
                        GitHubConnectorOp::CreateBranchProtection(new_protection),
                        format!("Create branch protection for {}/{} branch {}", owner, repo, branch)
                    ));
                }
                (Some(_), None) => {
                    res.push(connector_op!(
                        GitHubConnectorOp::DeleteBranchProtection,
                        format!("Delete branch protection for {}/{} branch {}", owner, repo, branch)
                    ));
                }
                (Some(current), Some(desired)) => {
                    if current != desired {
                        let old_protection: resource::BranchProtection = RON.from_str(&current?)?;
                        let new_protection: resource::BranchProtection = RON.from_str(&desired?)?;
                        let diff = diff_ron_values(&old_protection, &new_protection).unwrap_or_default();

                        res.push(connector_op!(
                            GitHubConnectorOp::UpdateBranchProtection(new_protection),
                            format!("Update branch protection for {}/{} branch {}\n{}", owner, repo, branch, diff)
                        ));
                    }
                }
            },
        }

        Ok(res)
    }
}
