//! Implementation of the load_spec command

use crate::cli::args::LoadSpecArgs;
use crate::types::responses::{FoundryResponse, LoadSpecResponse};
use anyhow::Result;

pub async fn execute(_args: LoadSpecArgs) -> Result<FoundryResponse<LoadSpecResponse>> {
    // TODO: Implement load_spec command
    Err(anyhow::anyhow!("load_spec command not yet implemented"))
}
