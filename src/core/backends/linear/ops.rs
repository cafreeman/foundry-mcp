use anyhow::Result;

use super::config::LinearConfig;
use super::graphql::LinearGraphQl;
use super::helpers::humanize_title;

// Pull in the registered schema named "linear"
#[cynic::schema("linear")]
mod schema {}

// ---- Teams ----
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Team")]
struct TeamLite {
    id: cynic::Id,
    name: String,
    key: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "TeamConnection")]
struct TeamConnectionLite {
    nodes: Vec<TeamLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct FindTeamsVars {
    first: Option<i32>,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "FindTeamsVars")]
struct FindTeams {
    #[cynic(rename = "teams")]
    teams: TeamConnectionLite,
}

async fn resolve_team_id(gql: &LinearGraphQl, cfg: &LinearConfig) -> Result<String> {
    if let Some(id) = cfg.team_id.clone() {
        return Ok(id);
    }

    let data = gql
        .execute(
            FindTeams::builder()
                .variables(FindTeamsVars { first: Some(100) })
                .build(),
        )
        .await?;

    if let Some(key) = cfg.team_key.as_ref() {
        if let Some(team) = data.teams.nodes.iter().find(|t| &t.key == key) {
            return Ok(team.id.to_string());
        }
    }
    if let Some(name) = cfg.team_name.as_ref() {
        if let Some(team) = data.teams.nodes.iter().find(|t| &t.name == name) {
            return Ok(team.id.to_string());
        }
    }

    Err(anyhow::anyhow!(
        "Unable to resolve Linear team id. Set LINEAR_TEAM_ID or provide LINEAR_TEAM_KEY/LINEAR_TEAM_NAME."
    ))
}

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

// ---- Documents ----
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Document")]
struct DocumentLite {
    id: cynic::Id,
    title: String,
    url: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DocumentConnection")]
struct DocumentConnectionLite {
    nodes: Vec<DocumentLite>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IDComparator")]
struct IdEqComparator {
    eq: Option<cynic::Id>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "ProjectFilter")]
struct ProjectFilterById {
    id: Option<IdEqComparator>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Project")]
struct ProjectWithDocuments {
    #[cynic(rename = "documents")]
    documents_conn: DocumentConnectionLite,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct ProjectDocumentsVars {
    filter: Option<ProjectFilterById>,
    first: Option<i32>,
    docs_first: Option<i32>,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "ProjectDocumentsVars")]
struct ProjectDocumentsQuery {
    #[cynic(rename = "projects")]
    projects_conn: ProjectConnectionForDocs,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ProjectConnection")]
struct ProjectConnectionForDocs {
    nodes: Vec<ProjectWithDocuments>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "DocumentCreateInput")]
struct DocumentCreateInputLinear {
    title: String,
    content: Option<String>,
    projectId: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DocumentPayload")]
struct DocumentPayloadLite {
    document: DocumentLite,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateDocumentVars {
    input: DocumentCreateInputLinear,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateDocumentVars")]
struct CreateDocumentOp {
    #[cynic(rename = "documentCreate")]
    document_create: DocumentPayloadLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "DocumentUpdateInput")]
struct DocumentUpdateInputLinear {
    title: Option<String>,
    content: Option<String>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct UpdateDocumentVars {
    id: String,
    input: DocumentUpdateInputLinear,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateDocumentVars")]
struct UpdateDocumentOp {
    #[cynic(rename = "documentUpdate")]
    document_update: DocumentPayloadLite,
}

/// Upsert the standard project documents (Vision and Tech Stack) by title.
pub async fn upsert_project_documents(
    gql: &LinearGraphQl,
    project_id: &str,
    project_name: &str,
    vision_md: &str,
    tech_stack_md: &str,
) -> Result<(String, String)> {
    // Load existing docs for the project (first page)
    let filter = ProjectFilterById {
        id: Some(IdEqComparator {
            eq: Some(cynic::Id::from(project_id.to_string())),
        }),
    };
    let pdq = ProjectDocumentsQuery::builder()
        .variables(ProjectDocumentsVars {
            filter: Some(filter),
            first: Some(1),
            docs_first: Some(50),
        })
        .build();
    let data = gql.execute(pdq).await?;

    let mut existing_vision: Option<DocumentLite> = None;
    let mut existing_tech: Option<DocumentLite> = None;

    if let Some(project) = data.projects_conn.nodes.into_iter().next() {
        for d in project.documents_conn.nodes.into_iter() {
            if d.title == "Vision" || d.title == format!("{} — Vision", project_name) {
                existing_vision = Some(d);
            } else if d.title == "Tech Stack" || d.title == format!("{} — Tech Stack", project_name)
            {
                existing_tech = Some(d);
            }
        }
    }

    // Compose content with hidden markers
    let project_marker = format!("<!-- foundry:project={}; v=1 -->\n", project_name);
    let vision_body = format!("{}{}", project_marker, vision_md);
    let tech_body = format!("{}{}", project_marker, tech_stack_md);

    // Upsert Vision
    let vision_id = if let Some(doc) = existing_vision {
        let _ = gql
            .execute(
                UpdateDocumentOp::builder()
                    .variables(UpdateDocumentVars {
                        id: doc.id.to_string(),
                        input: DocumentUpdateInputLinear {
                            title: None,
                            content: Some(vision_body.clone()),
                        },
                    })
                    .build(),
            )
            .await?;
        doc.id.to_string()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::builder()
                    .variables(CreateDocumentVars {
                        input: DocumentCreateInputLinear {
                            title: format!("{} — Vision", project_name),
                            content: Some(vision_body.clone()),
                            projectId: Some(project_id.to_string()),
                        },
                    })
                    .build(),
            )
            .await?;
        created.document_create.document.id.to_string()
    };

    // Upsert Tech Stack
    let tech_id = if let Some(doc) = existing_tech {
        let _ = gql
            .execute(
                UpdateDocumentOp::builder()
                    .variables(UpdateDocumentVars {
                        id: doc.id.to_string(),
                        input: DocumentUpdateInputLinear {
                            title: None,
                            content: Some(tech_body.clone()),
                        },
                    })
                    .build(),
            )
            .await?;
        doc.id.to_string()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::builder()
                    .variables(CreateDocumentVars {
                        input: DocumentCreateInputLinear {
                            title: format!("{} — Tech Stack", project_name),
                            content: Some(tech_body.clone()),
                            projectId: Some(project_id.to_string()),
                        },
                    })
                    .build(),
            )
            .await?;
        created.document_create.document.id.to_string()
    };

    Ok((vision_id, tech_id))
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
                    input: ProjectCreateInput {
                        name: name.to_string(),
                        description: description.map(|s| s.to_string()),
                        _phantom: None,
                    },
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
                    input: ProjectUpdateInput {
                        description: Some(description.to_string()),
                    },
                })
                .build(),
        )
        .await?;
    Ok(())
}

// ---- Labels ----
// We need to ensure the "foundry" label exists for our issues

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueLabel")]
struct IssueLabelLite {
    id: cynic::Id,
    name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelConnection")]
struct IssueLabelConnectionLite {
    nodes: Vec<IssueLabelLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct FindIssueLabelsVars {
    filter: Option<IssueLabelFilter>,
    first: Option<i32>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelFilter")]
struct IssueLabelFilter {
    name: Option<StringFilter>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "StringFilter")]
struct StringFilter {
    eq: Option<String>,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "FindIssueLabelsVars")]
struct FindIssueLabels {
    #[cynic(rename = "issueLabels")]
    issue_labels: IssueLabelConnectionLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelCreateInput")]
struct IssueLabelCreateInput {
    name: String,
    color: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelPayload")]
struct IssueLabelPayloadLite {
    issueLabel: Option<IssueLabelLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateIssueLabelVars {
    input: IssueLabelCreateInput,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateIssueLabelVars")]
struct CreateIssueLabelOp {
    #[cynic(rename = "issueLabelCreate")]
    issue_label_create: IssueLabelPayloadLite,
}

/// Find the "foundry" label or create it if it doesn't exist
pub async fn ensure_foundry_label(gql: &LinearGraphQl) -> Result<String> {
    // First try to find existing label
    let filter = IssueLabelFilter {
        name: Some(StringFilter {
            eq: Some("foundry".to_string()),
        }),
    };
    let data = gql
        .execute(
            FindIssueLabels::builder()
                .variables(FindIssueLabelsVars {
                    filter: Some(filter),
                    first: Some(50),
                })
                .build(),
        )
        .await?;

    if let Some(label) = data
        .issue_labels
        .nodes
        .into_iter()
        .find(|l| l.name == "foundry")
    {
        return Ok(label.id.to_string());
    }

    // Label doesn't exist, create it
    let created = gql
        .execute(
            CreateIssueLabelOp::builder()
                .variables(CreateIssueLabelVars {
                    input: IssueLabelCreateInput {
                        name: "foundry".to_string(),
                        color: Some("#4A90E2".to_string()),
                    },
                })
                .build(),
        )
        .await?;

    let label = created
        .issue_label_create
        .issueLabel
        .ok_or_else(|| anyhow::anyhow!("missing label in issueLabelCreate payload"))?;

    Ok(label.id.to_string())
}

// ---- Issues ----
// Phase C: Spec creation via Issues

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct IssueLite {
    id: cynic::Id,
    title: String,
    description: Option<String>,
    url: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssuePayload")]
struct IssuePayloadLite {
    issue: Option<IssueLite>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueCreateInput")]
struct IssueCreateInput {
    title: String,
    description: String,
    projectId: String,
    labelIds: Vec<String>,
    teamId: String,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateIssueVars {
    input: IssueCreateInput,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateIssueVars")]
struct CreateIssueOp {
    #[cynic(rename = "issueCreate")]
    issue_create: IssuePayloadLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueUpdateInput")]
struct IssueUpdateInput {
    description: Option<String>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct UpdateIssueVars {
    id: String,
    input: IssueUpdateInput,
}

#[derive(cynic::QueryBuilder, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateIssueVars")]
struct UpdateIssueOp {
    #[cynic(rename = "issueUpdate")]
    issue_update: IssuePayloadLite,
}

/// Create a Linear Issue for a spec with humanized title and hidden marker
pub async fn create_spec_issue(
    gql: &LinearGraphQl,
    cfg: &LinearConfig,
    project_id: &str,
    spec_name: &str,
    spec_content: &str,
    notes_url: &str,
) -> Result<(String, String)> {
    let foundry_label_id = ensure_foundry_label(gql).await?;

    // Humanize the spec name for the title
    let humanized_title = humanize_title(spec_name);

    // Create description with spec content + hidden marker + notes link
    let spec_marker = format!("<!-- foundry:specId={}; type=spec; v=1 -->\n", spec_name);
    let description = format!(
        "{}{}\n\n**Notes**: {}",
        spec_marker, spec_content, notes_url
    );

    // Resolve team id from config hints (id -> key -> name)
    let team_id = resolve_team_id(gql, cfg).await?;

    let created = gql
        .execute(
            CreateIssueOp::builder()
                .variables(CreateIssueVars {
                    input: IssueCreateInput {
                        title: humanized_title,
                        description: description.clone(),
                        projectId: project_id.to_string(),
                        labelIds: vec![foundry_label_id],
                        teamId: team_id,
                    },
                })
                .build(),
        )
        .await?;

    let issue = created
        .issue_create
        .issue
        .ok_or_else(|| anyhow::anyhow!("missing issue in issueCreate payload"))?;

    Ok((issue.id.to_string(), issue.url))
}

/// Update an issue's description
pub async fn update_issue_description(
    gql: &LinearGraphQl,
    issue_id: &str,
    description: &str,
) -> Result<()> {
    let _ = gql
        .execute(
            UpdateIssueOp::builder()
                .variables(UpdateIssueVars {
                    id: issue_id.to_string(),
                    input: IssueUpdateInput {
                        description: Some(description.to_string()),
                    },
                })
                .build(),
        )
        .await?;
    Ok(())
}

/// Create a document with the given title and content
pub async fn create_document(
    gql: &LinearGraphQl,
    title: &str,
    content: &str,
    project_id: &str,
) -> Result<DocumentLite> {
    let created = gql
        .execute(
            CreateDocumentOp::builder()
                .variables(CreateDocumentVars {
                    input: DocumentCreateInputLinear {
                        title: title.to_string(),
                        content: Some(content.to_string()),
                        projectId: Some(project_id.to_string()),
                    },
                })
                .build(),
        )
        .await?;

    Ok(created.document_create.document)
}
