use autoschematic_core::connector::ConnectorOp;
use serde::{Deserialize, Serialize};
use autoschematic_core::util::RON;

use crate::resource::{CollaboratorPrincipal, Role};

use super::resource::{GitHubRepository, BranchProtection};

#[derive(Debug, Serialize, Deserialize)]
pub enum GitHubConnectorOp {
    CreateRepository(GitHubRepository),
    UpdateRepository(GitHubRepository),
    DeleteRepository,

    CreateBranchProtection(BranchProtection),
    UpdateBranchProtection(BranchProtection),
    DeleteBranchProtection,

    AddCollaborator(CollaboratorPrincipal, Role),
    UpdateCollaborator(CollaboratorPrincipal, Role),
    RemoveCollaborator(CollaboratorPrincipal),
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
