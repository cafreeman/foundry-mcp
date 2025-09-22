//! Integration tests for Foundry CLI status commands across all environments
//!
//! These tests verify the status reporting functionality when querying all
//! supported environments (cursor and claude-code) simultaneously.

use anyhow::Result;

mod common;
use common::TestEnvironment;

/// Test status command for all environments
#[test]
fn test_status_all_environments() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Test status for all environments (no target specified)
        let status_response = env.get_status_response(None, false).await?;

        // Should return status for both claude-code and cursor
        assert_eq!(
            status_response.environments.len(),
            2,
            "Should return status for both environments"
        );

        let env_names: Vec<&String> = status_response
            .environments
            .iter()
            .map(|env| &env.name)
            .collect();
        assert!(
            env_names.contains(&&"claude-code".to_string()),
            "Should include claude-code"
        );
        assert!(
            env_names.contains(&&"cursor".to_string()),
            "Should include cursor"
        );

        // Neither should be installed initially
        for env_status in &status_response.environments {
            assert!(
                !env_status.installed,
                "No environments should be installed initially"
            );
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
