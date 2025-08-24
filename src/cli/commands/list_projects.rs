//! Implementation of the list_projects command

use crate::cli::args::ListProjectsArgs;
use crate::types::responses::{FoundryResponse, ListProjectsResponse};
use anyhow::Result;

pub async fn execute(_args: ListProjectsArgs) -> Result<FoundryResponse<ListProjectsResponse>> {
    // TODO: Implement list_projects command
    Err(anyhow::anyhow!("list_projects command not yet implemented"))
}
