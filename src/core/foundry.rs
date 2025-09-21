//! Foundry façade providing storage-agnostic domain logic

use crate::core::backends::FoundryBackend;
use crate::core::spec::SpecMatchStrategy;
use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecFileType, SpecMetadata},
};
use anyhow::Result;

/// Foundry façade providing storage-agnostic domain logic
pub struct Foundry<B: FoundryBackend> {
    backend: B,
}

impl<B: FoundryBackend> Foundry<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    // Project operations - thin delegation
    pub async fn create_project(&self, config: ProjectConfig) -> Result<Project> {
        self.backend.create_project(config).await
    }

    pub async fn project_exists(&self, name: &str) -> Result<bool> {
        self.backend.project_exists(name).await
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        self.backend.list_projects().await
    }

    pub async fn load_project(&self, name: &str) -> Result<Project> {
        self.backend.load_project(name).await
    }

    // Spec operations - thin delegation
    pub async fn create_spec(&self, config: SpecConfig) -> Result<Spec> {
        self.backend.create_spec(config).await
    }

    pub async fn list_specs(&self, project_name: &str) -> Result<Vec<SpecMetadata>> {
        self.backend.list_specs(project_name).await
    }

    pub async fn load_spec(&self, project_name: &str, spec_name: &str) -> Result<Spec> {
        self.backend.load_spec(project_name, spec_name).await
    }

    pub async fn update_spec_content(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        content: &str,
    ) -> Result<()> {
        self.backend
            .update_spec_content(project_name, spec_name, file_type, content)
            .await
    }

    pub async fn delete_spec(&self, project_name: &str, spec_name: &str) -> Result<()> {
        self.backend.delete_spec(project_name, spec_name).await
    }

    // Helper operations - thin delegation
    pub async fn get_latest_spec(&self, project_name: &str) -> Result<Option<SpecMetadata>> {
        self.backend.get_latest_spec(project_name).await
    }

    pub async fn count_specs(&self, project_name: &str) -> Result<usize> {
        self.backend.count_specs(project_name).await
    }

    // Domain logic - centralized here
    pub fn generate_spec_name(feature_name: &str) -> String {
        // This will be moved from spec.rs in Phase 1
        use chrono::{Datelike, Timelike, Utc};
        let now = Utc::now();
        format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}_{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            feature_name
        )
    }

    pub fn validate_spec_name(spec_name: &str) -> Result<()> {
        // This will be moved from spec.rs in Phase 1
        use crate::utils::timestamp;

        if timestamp::parse_spec_timestamp(spec_name).is_none() {
            return Err(anyhow::anyhow!(
                "Invalid spec name format. Expected: YYYYMMDD_HHMMSS_feature_name, got: {}",
                spec_name
            ));
        }

        // Validate feature name part
        if let Some(feature_name) = timestamp::extract_feature_name(spec_name) {
            if feature_name.is_empty() {
                return Err(anyhow::anyhow!(
                    "Spec name must include a feature name after the timestamp"
                ));
            }

            // Validate feature name follows snake_case convention
            if !feature_name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                || feature_name.starts_with('_')
                || feature_name.ends_with('_')
                || feature_name.contains("__")
            {
                return Err(anyhow::anyhow!(
                    "Feature name must be in snake_case format: {}",
                    feature_name
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Could not extract feature name from spec name: {}",
                spec_name
            ));
        }

        Ok(())
    }

    pub async fn find_spec_match(
        &self,
        project_name: &str,
        query: &str,
    ) -> Result<SpecMatchStrategy> {
        // This will be moved from spec.rs in Phase 1
        use strsim;

        // Validate inputs
        if query.trim().is_empty() {
            return Err(anyhow::anyhow!("Query cannot be empty"));
        }

        if project_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }

        let available_specs = self.list_specs(project_name).await?;

        if available_specs.is_empty() {
            return Ok(SpecMatchStrategy::None);
        }

        // Try exact spec name match first (highest priority)
        if let Some(exact_match) = available_specs.iter().find(|s| s.name == query) {
            return Ok(SpecMatchStrategy::Exact(exact_match.name.clone()));
        }

        // Try exact feature name match
        if let Some(feature_match) = available_specs.iter().find(|s| s.feature_name == query) {
            return Ok(SpecMatchStrategy::FeatureExact(feature_match.name.clone()));
        }

        // Try feature name substring match (case-insensitive)
        let query_lower = query.to_lowercase();
        let substring_matches: Vec<&SpecMetadata> = available_specs
            .iter()
            .filter(|s| s.feature_name.to_lowercase().contains(&query_lower))
            .collect();

        if substring_matches.len() == 1 {
            return Ok(SpecMatchStrategy::FeatureFuzzy(
                substring_matches[0].name.clone(),
            ));
        } else if substring_matches.len() > 1 {
            // Multiple substring matches - return for disambiguation
            let mut names: Vec<String> = substring_matches
                .into_iter()
                .map(|s| s.name.clone())
                .collect();
            names.sort();
            return Ok(SpecMatchStrategy::Multiple(names));
        }

        // Try fuzzy matching on feature names
        let feature_matches: Vec<(String, f32)> = available_specs
            .iter()
            .map(|s| {
                let similarity = strsim::normalized_levenshtein(query, &s.feature_name) as f32;
                (s.name.clone(), similarity)
            })
            .filter(|(_, confidence)| *confidence > 0.8) // High confidence threshold
            .collect();

        if feature_matches.len() == 1 {
            return Ok(SpecMatchStrategy::FeatureFuzzy(
                feature_matches[0].0.clone(),
            ));
        } else if feature_matches.len() > 1 {
            // Multiple feature matches - return for disambiguation
            let mut names: Vec<String> =
                feature_matches.into_iter().map(|(name, _)| name).collect();
            names.sort();
            return Ok(SpecMatchStrategy::Multiple(names));
        }

        // Try fuzzy matching on spec names
        let name_matches: Vec<(String, f32)> = available_specs
            .iter()
            .map(|s| {
                let similarity = strsim::normalized_levenshtein(query, &s.name) as f32;
                (s.name.clone(), similarity)
            })
            .filter(|(_, confidence)| *confidence > 0.8) // High confidence threshold
            .collect();

        if name_matches.len() == 1 {
            return Ok(SpecMatchStrategy::NameFuzzy(name_matches[0].0.clone()));
        } else if name_matches.len() > 1 {
            // Multiple name matches - return for disambiguation
            let mut names: Vec<String> = name_matches.into_iter().map(|(name, _)| name).collect();
            names.sort();
            return Ok(SpecMatchStrategy::Multiple(names));
        }

        Ok(SpecMatchStrategy::None)
    }

    // Edit commands integration - will be implemented in Phase 1
    // pub async fn apply_edit_commands(&self, project_name: &str, spec_name: &str,
    //                                commands: &[EditCommand]) -> Result<EditCommandsResult> {
    //     // This will be implemented in Phase 1
    //     todo!("Edit commands integration")
    // }
}

// SpecContentStore implementation will be added in Phase 1 when EditEngine integration is needed

/// Get the default Foundry instance with filesystem backend
///
/// Returns a Foundry instance using the FilesystemBackend as the default storage backend.
pub fn get_default_foundry() -> Result<Foundry<crate::core::backends::filesystem::FilesystemBackend>>
{
    let backend = crate::core::backends::filesystem::FilesystemBackend::new();
    Ok(Foundry::new(backend))
}
