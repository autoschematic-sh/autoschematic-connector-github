use crate::{GitHubConnector, addr::GitHubResourceAddress, op::GitHubConnectorOp, resource};
use autoschematic_core::{
    connector::{ConnectorOp, PlanResponseElement, ResourceAddress},
    connector_op,
    util::{RON, diff_ron_values},
};
use std::path::Path;

impl GitHubConnector {
    pub async fn do_plan(
        &self,
        addr: &Path,
        current: Option<Vec<u8>>,
        desired: Option<Vec<u8>>,
    ) -> anyhow::Result<Vec<PlanResponseElement>> {
        let addr = GitHubResourceAddress::from_path(addr)?;
        let mut res = Vec::new();

        match addr {
            GitHubResourceAddress::Config => {}
            GitHubResourceAddress::Repository { owner, repo } => match (current, desired) {
                (None, None) => {}
                (None, Some(desired_bytes)) => {
                    let desired_str = std::str::from_utf8(&desired_bytes)?;
                    let new_repo: resource::GitHubRepository = RON.from_str(desired_str)?;

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
                (Some(current_bytes), Some(desired_bytes)) => {
                    if current_bytes != desired_bytes {
                        let current_str = std::str::from_utf8(&current_bytes)?;
                        let desired_str = std::str::from_utf8(&desired_bytes)?;
                        let old_repo: resource::GitHubRepository = RON.from_str(current_str)?;
                        let new_repo: resource::GitHubRepository = RON.from_str(desired_str)?;

                        let diff = diff_ron_values(&old_repo, &new_repo).unwrap_or_default();
                        res.push(connector_op!(
                            GitHubConnectorOp::UpdateRepository(old_repo, new_repo),
                            format!("Update GitHub repository {}/{}\n{}", owner, repo, diff)
                        ));
                    }
                }
            },
            GitHubResourceAddress::BranchProtection { owner, repo, branch } => match (current, desired) {
                (None, None) => {}
                (None, Some(desired_bytes)) => {
                    let desired_str = std::str::from_utf8(&desired_bytes)?;
                    let new_protection: resource::BranchProtection = RON.from_str(desired_str)?;

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
                (Some(current_bytes), Some(desired_bytes)) => {
                    if current_bytes != desired_bytes {
                        let current_str = std::str::from_utf8(&current_bytes)?;
                        let desired_str = std::str::from_utf8(&desired_bytes)?;
                        let old_protection: resource::BranchProtection = RON.from_str(current_str)?;
                        let new_protection: resource::BranchProtection = RON.from_str(desired_str)?;
                        let diff = diff_ron_values(&old_protection, &new_protection).unwrap_or_default();

                        res.push(connector_op!(
                            GitHubConnectorOp::UpdateBranchProtection(old_protection, new_protection),
                            format!("Update branch protection for {}/{} branch {}\n{}", owner, repo, branch, diff)
                        ));
                    }
                }
            },
            GitHubResourceAddress::Collaborator { owner, repo, username } => match (current, desired) {
                (None, None) => {}
                (None, Some(desired_bytes)) => {
                    let desired_str = std::str::from_utf8(&desired_bytes)?;
                    let new_collaborator: resource::Collaborator = RON.from_str(desired_str)?;

                    res.push(connector_op!(
                        GitHubConnectorOp::AddCollaborator(new_collaborator),
                        format!("Add collaborator {} to {}/{}", username, owner, repo)
                    ));
                }
                (Some(_), None) => {
                    res.push(connector_op!(
                        GitHubConnectorOp::RemoveCollaborator,
                        format!("Remove collaborator {} from {}/{}", username, owner, repo)
                    ));
                }
                (Some(current_bytes), Some(desired_bytes)) => {
                    if current_bytes != desired_bytes {
                        let current_str = std::str::from_utf8(&current_bytes)?;
                        let desired_str = std::str::from_utf8(&desired_bytes)?;
                        let old_collaborator: resource::Collaborator = RON.from_str(current_str)?;
                        let new_collaborator: resource::Collaborator = RON.from_str(desired_str)?;

                        res.push(connector_op!(
                            GitHubConnectorOp::UpdateCollaboratorPermission(old_collaborator, new_collaborator),
                            format!("Update collaborator {} permissions for {}/{}", username, owner, repo)
                        ));
                    }
                }
            },
        }

        Ok(res)
    }
}
