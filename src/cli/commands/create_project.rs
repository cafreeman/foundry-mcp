//! Implementation of the create_project command

use crate::cli::args::CreateProjectArgs;
use crate::types::responses::{CreateProjectResponse, FoundryResponse};
use anyhow::Result;

pub async fn execute(_args: CreateProjectArgs) -> Result<FoundryResponse<CreateProjectResponse>> {
    // TODO: Implement create_project command
    Err(anyhow::anyhow!(
        "create_project command not yet implemented"
    ))
}
