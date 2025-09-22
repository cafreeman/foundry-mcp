use anyhow::Result;

use super::graphql::LinearGraphQl;

// Pull in the registered schema named "linear"
#[cynic::schema("linear")]
mod schema {}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Project")]
struct ProjectLite {
    id: cynic::Id,
    name: String,
    description: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ProjectConnection")]
struct ProjectConnectionLite {
    nodes: Vec<ProjectLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct FindProjectsVars {
    first: Option<i32>,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "FindProjectsVars")]
struct FindProjects {
    #[cynic(flatten)]
    projects: ProjectConnectionLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "ProjectCreateInput")]
struct ProjectCreateInput {
    name: String,
    description: Option<String>,
    #[allow(dead_code)]
    #[cynic(skip)]
    _phantom: Option<()>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ProjectPayload")]
struct ProjectPayloadLite {
    project: Option<ProjectLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateProjectVars {
    input: ProjectCreateInput,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateProjectVars")]
struct CreateProjectOp {
    #[cynic(rename = "projectCreate")]
    project_create: ProjectPayloadLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "ProjectUpdateInput")]
struct ProjectUpdateInput {
    description: Option<String>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct UpdateProjectVars {
    id: String,
    input: ProjectUpdateInput,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateProjectVars")]
struct UpdateProjectOp {
    #[cynic(rename = "projectUpdate")]
    project_update: ProjectPayloadLite,
}

/// Find an existing project by exact name, or create it with description.
pub async fn find_or_create_project(
    gql: &LinearGraphQl,
    name: &str,
    description: Option<&str>,
) -> Result<(String, String, Option<String>)> {
    // Small page query; filter locally by exact name
    let data = gql
        .execute(
            FindProjects::builder()
                .variables(FindProjectsVars { first: Some(25) })
                .build(),
        )
        .await?;

    if let Some(p) = data.projects.nodes.into_iter().find(|p| p.name == name) {
        return Ok((p.id.to_string(), p.name, p.description));
    }

    // Not found; create
    let created = gql
        .execute(
            CreateProjectOp::builder()
                .variables(CreateProjectVars {
                    input: ProjectCreateInput { name: name.to_string(), description: description.map(|s| s.to_string()), _phantom: None },
                })
                .build(),
        )
        .await?;

    let p = created
        .project_create
        .project
        .ok_or_else(|| anyhow::anyhow!("missing project in projectCreate payload"))?;

    Ok((p.id.to_string(), p.name, p.description))
}

/// Update a project's description by id
pub async fn upsert_project_description(
    gql: &LinearGraphQl,
    id: &str,
    description: &str,
) -> Result<()> {
    let _ = gql
        .execute(
            UpdateProjectOp::builder()
                .variables(UpdateProjectVars {
                    id: id.to_string(),
                    input: ProjectUpdateInput { description: Some(description.to_string()) },
                })
                .build(),
        )
        .await?;
    Ok(())
}