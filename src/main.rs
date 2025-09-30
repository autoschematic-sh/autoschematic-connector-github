use autoschematic_core::tarpc_bridge::tarpc_connector_main;
use connector::GitHubConnector;

pub mod addr;
pub mod client;
pub mod config;
pub mod connector;
pub mod github_ext;
pub mod op;
pub mod resource;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tarpc_connector_main::<GitHubConnector>().await?;
    Ok(())
}
