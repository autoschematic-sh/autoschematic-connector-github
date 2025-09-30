use autoschematic_core::connector::ConnectorOp;
use serde::{Deserialize, Serialize};
use autoschematic_core::util::RON;

use super::resource::{GitHubRepository, BranchProtection, Collaborator};

#[derive(Debug, Serialize, Deserialize)]
pub enum GitHubConnectorOp {
    // Repository operations
    CreateRepository(GitHubRepository),
    UpdateRepository(GitHubRepository, GitHubRepository), // (old, new)
    DeleteRepository,

    // Branch protection operations
    CreateBranchProtection(BranchProtection),
    UpdateBranchProtection(BranchProtection, BranchProtection), // (old, new)
    DeleteBranchProtection,

    // Collaborator operations
    AddCollaborator(Collaborator),
    UpdateCollaboratorPermission(Collaborator, Collaborator), // (old, new)
    RemoveCollaborator,
}

impl ConnectorOp for GitHubConnectorOp {
    fn to_string(&self) -> Result<String, anyhow::Error> {
        Ok(RON.to_string(self)?)
    }

    fn from_str(s: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(RON.from_str(s)?)
    }
}
