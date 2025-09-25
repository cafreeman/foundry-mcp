use anyhow::Result;
use cynic::{QueryBuilder, MutationBuilder};

use super::config::LinearConfig;
use super::graphql::LinearGraphQl;
use super::helpers::humanize_title;

#[cfg(feature = "linear_backend")]
use crate::linear_reconcile::ExistingSubIssue;

// Pull in the registered schema named "linear"
#[cynic::schema("linear")]
mod schema {}

// Configure scalar mapping for DateTime
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "DateTime")]
pub struct DateTime(String);

// Define local payload types for mutations
#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueArchivePayload")]
struct IssueArchivePayload {
    success: bool,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DocumentArchivePayload")]
struct DocumentArchivePayload {
    success: bool,
}

// Local input type definitions for mutations
#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "StringComparator")]
struct StringComparator {
    eq: Option<String>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelFilter")]
struct IssueLabelFilter {
    name: Option<StringComparator>,
}

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

#[derive(cynic::QueryFragment, Debug, Clone)]
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
            FindTeams::build(FindTeamsVars { first: Some(100) }),
        )
        .await?;

    if let Some(key) = cfg.team_key.as_ref() {
        if let Some(team) = data.teams.nodes.iter().find(|t| &t.key == key) {
            return Ok(team.id.clone().into_inner());
        }
    }
    if let Some(name) = cfg.team_name.as_ref() {
        if let Some(team) = data.teams.nodes.iter().find(|t| &t.name == name) {
            return Ok(team.id.clone().into_inner());
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

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "FindProjectsVars")]
struct FindProjects {
    #[cynic(rename = "projects")]
    projects: ProjectConnectionLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "ProjectCreateInput")]
struct ProjectCreateInput {
    name: String,
    description: Option<String>,
    #[cynic(rename = "teamIds")]
    team_ids: Vec<String>,
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

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateProjectVars")]
struct CreateProjectOp {
    #[arguments(input: $input)]
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

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateProjectVars")]
struct UpdateProjectOp {
    #[arguments(id: $id, input: $input)]
    #[cynic(rename = "projectUpdate")]
    project_update: ProjectPayloadLite,
}

// ---- Documents ----
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Document")]
pub struct DocumentLite {
    pub id: cynic::Id,
    pub title: String,
    pub url: String,
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

#[derive(cynic::QueryFragment, Debug, Clone)]
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
    #[cynic(rename = "projectId")]
    project_id: Option<String>,
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

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateDocumentVars")]
struct CreateDocumentOp {
    #[arguments(input: $input)]
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

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateDocumentVars")]
struct UpdateDocumentOp {
    #[arguments(id: $id, input: $input)]
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
    let pdq = ProjectDocumentsQuery::build(ProjectDocumentsVars {
        filter: Some(filter),
        first: Some(1),
        docs_first: Some(50),
    });
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
                UpdateDocumentOp::build(UpdateDocumentVars {
                    id: doc.id.clone().into_inner(),
                    input: DocumentUpdateInputLinear {
                        title: None,
                        content: Some(vision_body.clone()),
                    },
                }),
            )
            .await?;
        doc.id.clone().into_inner()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::build(CreateDocumentVars {
                    input: DocumentCreateInputLinear {
                        title: format!("{} — Vision", project_name),
                        content: Some(vision_body.clone()),
                        project_id: Some(project_id.to_string()),
                    },
                }),
            )
            .await?;
        created.document_create.document.id.clone().into_inner()
    };

    // Upsert Tech Stack
    let tech_id = if let Some(doc) = existing_tech {
        let _ = gql
            .execute(
                UpdateDocumentOp::build(UpdateDocumentVars {
                    id: doc.id.clone().into_inner(),
                    input: DocumentUpdateInputLinear {
                        title: None,
                        content: Some(tech_body.clone()),
                    },
                }),
            )
            .await?;
        doc.id.clone().into_inner()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::build(CreateDocumentVars {
                    input: DocumentCreateInputLinear {
                        title: format!("{} — Tech Stack", project_name),
                        content: Some(tech_body.clone()),
                        project_id: Some(project_id.to_string()),
                    },
                }),
            )
            .await?;
        created.document_create.document.id.clone().into_inner()
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
            FindProjects::build(FindProjectsVars { first: Some(25) })
        )
        .await?;

    if let Some(p) = data.projects.nodes.into_iter().find(|p| p.name == name) {
        return Ok((p.id.clone().into_inner(), p.name, p.description));
    }

    // Not found; create
    let created = gql
        .execute(
            CreateProjectOp::build(CreateProjectVars {
                    input: ProjectCreateInput {
                        name: name.to_string(),
                        description: description.map(|s| s.to_string()),
                        team_ids: vec![], // TODO: Pass team IDs from caller
                    },
                }),
        )
        .await?;

    let p = created
        .project_create
        .project
        .ok_or_else(|| anyhow::anyhow!("missing project in projectCreate payload"))?;

    Ok((p.id.clone().into_inner(), p.name, p.description))
}

/// Update a project's description by id
pub async fn upsert_project_description(
    gql: &LinearGraphQl,
    id: &str,
    description: &str,
) -> Result<()> {
    let _ = gql
        .execute(
            UpdateProjectOp::build(UpdateProjectVars {
                    id: id.to_string(),
                    input: ProjectUpdateInput {
                        description: Some(description.to_string()),
                    },
                }),
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
#[cynic(graphql_type = "IssueLabelCollectionFilter")]
struct IssueLabelCollectionFilter {
    name: Option<StringComparator>,
}



#[derive(cynic::QueryFragment, Debug, Clone)]
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
    #[cynic(rename = "issueLabel")]
    issue_label: Option<IssueLabelLite>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateIssueLabelVars {
    input: IssueLabelCreateInput,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateIssueLabelVars")]
struct CreateIssueLabelOp {
    #[arguments(input: $input)]
    #[cynic(rename = "issueLabelCreate")]
    issue_label_create: IssueLabelPayloadLite,
}

/// Find the "foundry" label or create it if it doesn't exist
pub async fn ensure_foundry_label(gql: &LinearGraphQl) -> Result<String> {
    // First try to find existing label
    let filter = IssueLabelFilter {
        name: Some(StringComparator {
            eq: Some("foundry".to_string()),
        }),
    };
    let data = gql
        .execute(
            FindIssueLabels::build(FindIssueLabelsVars {
                    filter: Some(filter),
                    first: Some(50),
                }),
        )
        .await?;

    if let Some(label) = data
        .issue_labels
        .nodes
        .into_iter()
        .find(|l| l.name == "foundry")
    {
        return Ok(label.id.clone().into_inner());
    }

    // Label doesn't exist, create it
    let created = gql
        .execute(
            CreateIssueLabelOp::build(CreateIssueLabelVars {
                    input: IssueLabelCreateInput {
                        name: "foundry".to_string(),
                        color: Some("#4A90E2".to_string()),
                    },
                }),
        )
        .await?;

    let label = created
        .issue_label_create
        .issue_label
        .ok_or_else(|| anyhow::anyhow!("missing label in issueLabelCreate payload "))?;

    Ok(label.id.clone().into_inner())
}

// ---- Issues ----
// Phase C: Spec creation via Issues

#[allow(dead_code)]
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
    #[cynic(rename = "projectId")]
    project_id: String,
    #[cynic(rename = "labelIds")]
    label_ids: Vec<String>,
    #[cynic(rename = "teamId")]
    team_id: String,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateIssueVars {
    input: IssueCreateInput,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateIssueVars")]
struct CreateIssueOp {
    #[arguments(input: $input)]
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

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateIssueVars")]
struct UpdateIssueOp {
    #[arguments(id: $id, input: $input)]
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
        "{}{}

**Notes**: {}",
        spec_marker, spec_content, notes_url
    );

    // Resolve team id from config hints (id -> key -> name)
    let team_id = resolve_team_id(gql, cfg).await?;

    let created = gql
        .execute(
            CreateIssueOp::build(CreateIssueVars {
                    input: IssueCreateInput {
                        title: humanized_title,
                        description: description.clone(),
                        project_id: project_id.to_string(),
                        label_ids: vec![foundry_label_id],
                        team_id: team_id,
                    },
                }),
        )
        .await?;

    let issue = created
        .issue_create
        .issue
        .ok_or_else(|| anyhow::anyhow!("missing issue in issueCreate payload"))?;

    Ok((issue.id.clone().into_inner(), issue.url))
}

/// Update an issue's description
pub async fn update_issue_description(
    gql: &LinearGraphQl,
    issue_id: &str,
    description: &str,
) -> Result<()> {
    let _ = gql
        .execute(
            UpdateIssueOp::build(UpdateIssueVars {
                    id: issue_id.to_string(),
                    input: IssueUpdateInput {
                        description: Some(description.to_string()),
                    },
                }),
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
            CreateDocumentOp::build(CreateDocumentVars {
                    input: DocumentCreateInputLinear {
                        title: title.to_string(),
                        content: Some(content.to_string()),
                        project_id: Some(project_id.to_string()),
                    },
                }),
        )
        .await?;

    Ok(created.document_create.document)
}

// ---- Sub-issues (Phase D) ----
// Feature-gated to avoid breaking builds until mutations/queries are finalized
// ---- Sub-Issues (Phase D/E) ----

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct SubIssueLite {
    id: cynic::Id,
    title: String,
    description: Option<String>,
    #[cynic(rename = "state")]
    state: WorkflowState,
    labels: IssueLabelConnectionLite,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "WorkflowState")]
struct WorkflowState {
    #[cynic(rename = "type")]
    type_name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
struct SubIssueConnectionLite {
    nodes: Vec<SubIssueLite>,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct IssueWithSubIssues {
    id: cynic::Id,
    #[cynic(rename = "children")]
    children: SubIssueConnectionLite,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IDComparator")]
struct IssueIdComparator {
    eq: Option<cynic::Id>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueFilter")]
struct IssueFilterById {
    id: Option<IssueIdComparator>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
struct IssueConnectionForSubIssues {
    nodes: Vec<IssueWithSubIssues>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct ListSubIssuesVars {
    filter: Option<IssueFilterById>,
    first: Option<i32>,
    children_first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "ListSubIssuesVars")]
struct ListSubIssuesQuery {
    #[cynic(rename = "issues")]
    issues: IssueConnectionForSubIssues,
}

#[cfg(feature = "linear_backend")]
#[allow(dead_code)]
pub async fn list_sub_issues_for_parent(
    gql: &LinearGraphQl,
    parent_issue_id: &str,
) -> Result<Vec<(String, String, bool, bool, Option<String>)>> {
    // Returns tuples: (id, title, open, has_foundry_label, task_key)
    use crate::core::backends::linear::helpers::parse_foundry_task_key_marker;

    let data = gql
        .execute(
            ListSubIssuesQuery::build(ListSubIssuesVars {
                    filter: Some(IssueFilterById {
                        id: Some(IssueIdComparator {
                            eq: Some(cynic::Id::new(parent_issue_id)),
                        }),
                    }),
                    first: Some(1),
                    children_first: Some(100), // Get all sub-issues
                }),
        )
        .await?;

    let parent_issue = data
        .issues
        .nodes
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Parent issue not found: {}", parent_issue_id))?;

    let mut results = Vec::new();
    for sub_issue in parent_issue.children.nodes {
        let id = sub_issue.id.clone().into_inner();
        let title = sub_issue.title;
        let open =
            sub_issue.state.type_name != "completed" && sub_issue.state.type_name != "canceled";

        // Check for foundry label
        let has_foundry_label = sub_issue
            .labels
            .nodes
            .iter()
            .any(|label| label.name == "foundry");

        // Parse taskKey from description if available
        let task_key = sub_issue
            .description
            .and_then(|desc| parse_foundry_task_key_marker(&desc));

        results.push((id, title, open, has_foundry_label, task_key));
    }

    Ok(results)
}

// ---- Sub-issue Creation ----
#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueCreateInput")]
struct SubIssueCreateInput {
    title: String,
    description: String,
    #[cynic(rename = "projectId")]
    project_id: String,
    #[cynic(rename = "labelIds")]
    label_ids: Vec<String>,
    #[cynic(rename = "teamId")]
    team_id: String,
    #[cynic(rename = "parentId")]
    parent_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct CreateSubIssueVars {
    input: SubIssueCreateInput,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "CreateSubIssueVars")]
struct CreateSubIssueOp {
    #[arguments(input: $input)]
    #[cynic(rename = "issueCreate")]
    issue_create: IssuePayloadLite,
}

#[cfg(feature = "linear_backend")]
#[allow(dead_code)]
pub async fn create_sub_issue_with_marker(
    gql: &LinearGraphQl,
    cfg: &LinearConfig,
    parent_issue_id: &str,
    project_id: &str,
    title: &str,
    spec_id: &str,
    task_key: &str,
) -> Result<String> {
    // Compose hidden marker in the body
    let marker = format!(
        "<!-- foundry:specId={}; type=task; v=1; taskKey={} -->\n",
        spec_id, task_key
    );
    let description = format!("{}{}", marker, title);

    let foundry_label_id = ensure_foundry_label(gql).await?;
    let team_id = resolve_team_id(gql, cfg).await?;

    let created = gql
        .execute(
            CreateSubIssueOp::build(CreateSubIssueVars {
                    input: SubIssueCreateInput {
                        title: title.to_string(),
                        description,
                        project_id: project_id.to_string(),
                        label_ids: vec![foundry_label_id],
                        team_id: team_id,
                        parent_id: Some(parent_issue_id.to_string()),
                    },
                }),
        )
        .await?;

    let issue = created
        .issue_create
        .issue
        .ok_or_else(|| anyhow::anyhow!("missing issue in issueCreate payload"))?;

    Ok(issue.id.clone().into_inner())
}

// ---- Issue State Management ----
#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueUpdateInput")]
struct IssueStateUpdateInput {
    #[cynic(rename = "stateId")]
    state_id: Option<String>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct UpdateIssueStateVars {
    id: String,
    input: IssueStateUpdateInput,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateIssueStateVars")]
struct UpdateIssueStateOp {
    #[arguments(id: $id, input: $input)]
    #[cynic(rename = "issueUpdate")]
    issue_update: IssuePayloadLite,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "WorkflowState")]
struct WorkflowStateFull {
    id: cynic::Id,
    name: String,
    #[cynic(rename = "type")]
    type_name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "WorkflowStateConnection")]
struct WorkflowStateConnection {
    nodes: Vec<WorkflowStateFull>,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Team")]
struct TeamWithWorkflow {
    id: cynic::Id,
    states: WorkflowStateConnection,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "TeamConnection")]
struct TeamConnectionWithWorkflow {
    nodes: Vec<TeamWithWorkflow>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct FindTeamStatesVars {
    first: Option<i32>,
    states_first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "FindTeamStatesVars")]
struct FindTeamStatesQuery {
    #[cynic(rename = "teams")]
    teams: TeamConnectionWithWorkflow,
}

async fn find_workflow_state_by_type(gql: &LinearGraphQl, state_type: &str) -> Result<String> {
    let data = gql
        .execute(
            FindTeamStatesQuery::build(FindTeamStatesVars {
                    first: Some(10),
                    states_first: Some(50),
                }),
        )
        .await?;

    for team in data.teams.nodes {
        for state in team.states.nodes {
            if state.type_name == state_type {
                return Ok(state.id.clone().into_inner());
            }
        }
    }

    Err(anyhow::anyhow!(
        "No workflow state found with type '{}'",
        state_type
    ))
}

#[cfg(feature = "linear_backend")]
#[allow(dead_code)]
pub async fn close_issue(gql: &LinearGraphQl, issue_id: &str) -> Result<()> {
    let completed_state_id = find_workflow_state_by_type(gql, "completed").await?;

    let _ = gql
        .execute(
            UpdateIssueStateOp::build(UpdateIssueStateVars {
                    id: issue_id.to_string(),
                    input: IssueStateUpdateInput {
                        state_id: Some(completed_state_id),
                    },
                }),
        )
        .await?;

    Ok(())
}

#[cfg(feature = "linear_backend")]
#[allow(dead_code)]
pub async fn reopen_issue(gql: &LinearGraphQl, issue_id: &str) -> Result<()> {
    let in_progress_state_id = find_workflow_state_by_type(gql, "started").await?;

    let _ = gql
        .execute(
            UpdateIssueStateOp::build(UpdateIssueStateVars {
                    id: issue_id.to_string(),
                    input: IssueStateUpdateInput {
                        state_id: Some(in_progress_state_id),
                    },
                }),
        )
        .await?;

    Ok(())
}

// ---- Issue Label Management ----
#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct IssueWithLabels {
    id: cynic::Id,
    labels: IssueLabelConnectionLite,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
struct IssueConnectionForLabels {
    nodes: Vec<IssueWithLabels>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct GetIssueLabelVars {
    filter: Option<IssueFilterById>,
    first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "GetIssueLabelVars")]
struct GetIssueLabelsQuery {
    #[cynic(rename = "issues")]
    issues: IssueConnectionForLabels,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueUpdateInput")]
struct IssueLabelUpdateInput {
    #[cynic(rename = "labelIds")]
    label_ids: Option<Vec<String>>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct UpdateIssueLabelVars {
    id: String,
    input: IssueLabelUpdateInput,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "UpdateIssueLabelVars")]
struct UpdateIssueLabelOp {
    #[arguments(id: $id, input: $input)]
    #[cynic(rename = "issueUpdate")]
    issue_update: IssuePayloadLite,
}

#[cfg(feature = "linear_backend")]
#[allow(dead_code)]
pub async fn ensure_label_on_issue(
    gql: &LinearGraphQl,
    issue_id: &str,
    label_id: &str,
) -> Result<()> {
    // First check if the issue already has the label
    let data = gql
        .execute(
            GetIssueLabelsQuery::build(GetIssueLabelVars {
                    filter: Some(IssueFilterById {
                        id: Some(IssueIdComparator {
                            eq: Some(cynic::Id::new(issue_id)),
                        }),
                    }),
                    first: Some(1),
                }),
        )
        .await?;

    let issue = data
        .issues
        .nodes
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Issue not found: {}", issue_id))?;

    // Check if the label is already attached
    if issue
        .labels
        .nodes
        .iter()
        .any(|label| label.id.clone().into_inner() == label_id)
    {
        return Ok(()); // Label already attached
    }

    // Collect existing label IDs and add the new one
    let mut label_ids: Vec<String> = issue
        .labels
        .nodes
        .iter()
        .map(|label| label.id.clone().into_inner())
        .collect();
    label_ids.push(label_id.to_string());

    // Update the issue with the new label list
    let _ = gql
        .execute(
            UpdateIssueLabelOp::build(UpdateIssueLabelVars {
                    id: issue_id.to_string(),
                    input: IssueLabelUpdateInput {
                        label_ids: Some(label_ids),
                    },
                }),
        )
        .await?;

    Ok(())
}

// ---- Spec Listing & Management (Phase E) ----

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct SpecIssueLite {
    id: cynic::Id,
    title: String,
    description: Option<String>,
    url: String,
    #[cynic(rename = "createdAt")]
    created_at: DateTime,
    labels: IssueLabelConnectionLite,
    #[cynic(rename = "project")]
    project_ref: Option<ProjectLite>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
struct SpecIssueConnection {
    nodes: Vec<SpecIssueLite>,
    #[cynic(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "PageInfo")]
struct PageInfo {
    #[cynic(rename = "hasNextPage")]
    has_next_page: bool,
    #[cynic(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueLabelFilter")]
struct FoundryLabelFilter {
    name: Option<StringComparator>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "IssueFilter")]
struct IssueFilterForSpecs {
    labels: Option<IssueLabelCollectionFilter>,
    project: Option<NullableProjectFilter>,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "NullableProjectFilter")]
struct NullableProjectFilter {
    name: Option<StringComparator>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct ListSpecIssuesVars {
    filter: Option<IssueFilterForSpecs>,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "ListSpecIssuesVars")]
struct ListSpecIssuesQuery {
    #[cynic(rename = "issues")]
    issues: SpecIssueConnection,
}

#[cfg(feature = "linear_backend")]
pub async fn list_spec_issues_by_project(
    gql: &LinearGraphQl,
    project_name: &str,
    after_cursor: Option<String>,
    limit: Option<i32>,
) -> Result<(Vec<crate::types::spec::SpecMetadata>, bool, Option<String>)> {
    use crate::core::backends::linear::helpers::parse_foundry_spec_marker;

    let data = gql
        .execute(
            ListSpecIssuesQuery::build(ListSpecIssuesVars {
                    filter: Some(IssueFilterForSpecs {
    labels: Some(IssueLabelCollectionFilter {
        name: Some(StringComparator {
            eq: Some("foundry".to_string()),
        }),
    }),
    project: Some(NullableProjectFilter {
        name: Some(StringComparator {
            eq: Some(project_name.to_string()),
        }),
    }),
                    }),
                    first: limit.or(Some(50)),
                    after: after_cursor,
                }),
        )
        .await?;

    let mut specs = Vec::new();

    for issue in data.issues.nodes {
        // Parse spec marker from description to get canonical spec_name
        if let Some(description) = &issue.description {
            if let Ok(Some(spec_id)) = parse_foundry_spec_marker(description) {
                let _locator = crate::core::backends::ResourceLocator::Linear {
                    project_id: issue
                        .project_ref
                        .as_ref()
                        .map(|p| p.id.clone().into_inner())
                        .unwrap_or_default(),
                    issue_id: issue.id.clone().into_inner(),
                    notes_document_id: String::new(), // Would need separate query to get this
                    issue_url: issue.url.clone(),
                    notes_url: String::new(),
                };

                specs.push(crate::types::spec::SpecMetadata {
                    name: spec_id,
                    feature_name: issue.title,
                    created_at: issue.created_at.0,
                    project_name: project_name.to_string(),
                });
            }
        }
    }

    let has_next_page = data.issues.page_info.has_next_page;
    let next_cursor = data.issues.page_info.end_cursor;

    Ok((specs, has_next_page, next_cursor))
}

// ---- Load Spec Content (Phase E) ----

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Issue")]
struct SpecIssueForLoad {
    id: cynic::Id,
    title: String,
    description: Option<String>,
    url: String,
    #[cynic(rename = "createdAt")]
    created_at: DateTime,
    #[cynic(rename = "project")]
    project_ref: Option<ProjectLite>,
    #[cynic(rename = "children")]
    children: SubIssueConnectionLite,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IssueConnection")]
struct SpecIssueForLoadConnection {
    nodes: Vec<SpecIssueForLoad>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
struct LoadSpecIssueVars {
    filter: Option<IssueFilterForSpecs>,
    first: Option<i32>,
    children_first: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "LoadSpecIssueVars")]
struct LoadSpecIssueQuery {
    #[cynic(rename = "issues")]
    issues: SpecIssueForLoadConnection,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Document")]
struct DocumentForLoad {
    id: cynic::Id,
    title: String,
    content: Option<String>,
    url: String,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "DocumentConnection")]
struct DocumentForLoadConnection {
    nodes: Vec<DocumentForLoad>,
}

#[allow(dead_code)]
#[derive(cynic::QueryVariables, Debug, Clone)]
struct LoadNotesDocumentVars {
    filter: Option<ProjectFilterById>,
    first: Option<i32>,
    docs_first: Option<i32>,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "LoadNotesDocumentVars")]
struct LoadNotesDocumentQuery {
    #[cynic(rename = "projects")]
    projects_conn: ProjectConnectionForDocs,
}

#[cfg(feature = "linear_backend")]
pub async fn load_spec_by_marker(
    gql: &LinearGraphQl,
    project_name: &str,
    spec_name: &str,
) -> Result<Option<crate::types::spec::Spec>> {
    use crate::core::backends::ResourceLocator;
    use crate::core::backends::linear::helpers::parse_foundry_spec_marker;

    // 1. Find the spec issue by project and foundry label
    let data = gql
        .execute(
            LoadSpecIssueQuery::build(LoadSpecIssueVars {
                    filter: Some(IssueFilterForSpecs {
    labels: Some(IssueLabelCollectionFilter {
        name: Some(StringComparator {
            eq: Some("foundry".to_string()),
        }),
    }),
    project: Some(NullableProjectFilter {
        name: Some(StringComparator {
            eq: Some(project_name.to_string()),
        }),
    }),
                    }),
                    first: Some(100),          // Get all potential spec issues
                    children_first: Some(100), // Get all sub-issues
                }),
        )
        .await?;

    // 2. Find the issue with matching spec_name in marker
    let mut target_issue: Option<SpecIssueForLoad> = None;

    for issue in data.issues.nodes {
        if let Some(description) = &issue.description {
            if let Ok(Some(found_spec_id)) = parse_foundry_spec_marker(description) {
                if found_spec_id == spec_name {
                    target_issue = Some(issue);
                    break;
                }
            }
        }
    }

    let issue = match target_issue {
        Some(issue) => issue,
        None => return Ok(None), // Spec not found
    };

    // 3. Parse spec content from issue description (remove marker)
    let spec_content = if let Some(desc) = &issue.description {
        // Remove the hidden marker from content
        if let Some(marker_start) = desc.find("<!--") {
            if let Some(marker_end) = desc[marker_start..].find("-->") {
                let full_marker_end = marker_start + marker_end + 3;
                let before = &desc[..marker_start];
                let after = &desc[full_marker_end..];
                format!("{}{}", before, after).trim().to_string()
            } else {
                desc.clone()
            }
        } else {
            desc.clone()
        }
    } else {
        String::new()
    };

    // 4. Build tasks from sub-issues
    let mut tasks = Vec::new();
    for sub_issue in issue.children.nodes {
        let checked = sub_issue.state.type_name == "completed";
        let task_line = if checked {
            format!("- [x] {}", sub_issue.title)
        } else {
            format!("- [ ] {}", sub_issue.title)
        };
        tasks.push(task_line);
    }
    let tasks_content = if tasks.is_empty() {
        String::new()
    } else {
        tasks.join("\n")
    };

    // 5. TODO: Load notes document (would need separate query by project docs)
    let notes_content = String::new(); // Placeholder for now

    // 6. Build ResourceLocator
    let locator = ResourceLocator::Linear {
        project_id: issue
            .project_ref
            .as_ref()
            .map(|p| p.id.clone().into_inner())
            .unwrap_or_default(),
        issue_id: issue.id.clone().into_inner(),
        notes_document_id: String::new(), // Would get from notes query
        issue_url: issue.url.clone(),
        notes_url: String::new(), // Would get from notes query
    };

    Ok(Some(crate::types::spec::Spec {
        name: spec_name.to_string(),
        created_at: issue.created_at.0,
        path: std::path::PathBuf::from(format!("/specs/{}", spec_name)), // Placeholder path for compatibility
        project_name: project_name.to_string(),
        location_hint: Some(issue.url),
        locator: Some(locator),
        content: crate::types::spec::SpecContentData {
            spec: spec_content,
            tasks: tasks_content,
            notes: notes_content,
        },
    }))
}

// ---- Delete/Archive Operations (Phase E) ----

#[derive(cynic::QueryVariables, Debug, Clone)]
struct ArchiveIssueVars {
    id: String,
    trash: Option<bool>,
}

// Future feature: issue archiving
#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "ArchiveIssueVars")]
struct ArchiveIssueOp {
    #[arguments(id: $id, trash: $trash)]
    #[cynic(rename = "issueArchive")]
    issue_archive: IssueArchivePayload,
}



// Future feature: document deletion
#[allow(dead_code)]
#[derive(cynic::QueryVariables, Debug, Clone)]
struct DeleteDocumentVars {
    id: String,
}

#[allow(dead_code)]
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Mutation", variables = "DeleteDocumentVars")]
struct DeleteDocumentOp {
    #[arguments(id: $id)]
    #[cynic(rename = "documentDelete")]
    document_delete: DocumentArchivePayload,
}



#[cfg(feature = "linear_backend")]
pub async fn delete_spec_by_marker(
    gql: &LinearGraphQl,
    project_name: &str,
    spec_name: &str,
) -> Result<()> {
    use crate::core::backends::linear::helpers::parse_foundry_spec_marker;

    // 1. Find the spec issue
    let data = gql
        .execute(
            LoadSpecIssueQuery::build(LoadSpecIssueVars {
                    filter: Some(IssueFilterForSpecs {
    labels: Some(IssueLabelCollectionFilter {
        name: Some(StringComparator {
            eq: Some("foundry".to_string()),
        }),
    }),
    project: Some(NullableProjectFilter {
        name: Some(StringComparator {
            eq: Some(project_name.to_string()),
        }),
    }),
                    }),
                    first: Some(100),
                    children_first: Some(100), // Need sub-issues to close them
                }),
        )
        .await?;

    // 2. Find the target issue with matching spec marker
    let mut target_issue: Option<SpecIssueForLoad> = None;

    for issue in data.issues.nodes {
        if let Some(description) = &issue.description {
            if let Ok(Some(found_spec_id)) = parse_foundry_spec_marker(description) {
                if found_spec_id == spec_name {
                    target_issue = Some(issue);
                    break;
                }
            }
        }
    }

    let issue = match target_issue {
        Some(issue) => issue,
        None => {
            return Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            ));
        }
    };

    // 3. Close/archive all sub-issues first
    for sub_issue in &issue.children.nodes {
        // Close the sub-issue (we already have close_issue implemented)
        close_issue(gql, &sub_issue.id.clone().into_inner()).await?;
    }

    // 4. Archive the main spec issue
    let _ = gql
        .execute(
            ArchiveIssueOp::build(ArchiveIssueVars {
                    id: issue.id.clone().into_inner(),
                    trash: Some(false), // Archive, don't permanently delete
                }),
        )
        .await?;

    // 5. TODO: Delete notes document if we can identify it
    // This would require finding the notes document by marker or title pattern
    // For now, we'll leave this as a placeholder since we didn't implement
    // notes document loading in load_spec_by_marker yet

    Ok(())
}

// Helper to convert listed sub-issues to reconciliation inputs
#[cfg(feature = "linear_backend")]
pub(crate) fn build_existing_from_tuples(
    rows: Vec<(String, String, bool, bool, Option<String>)>,
) -> Vec<ExistingSubIssue> {
    rows.into_iter()
        .map(|(id, title, open, has_foundry_label, task_key)| {
            ExistingSubIssue {
                id,
                title,
                open,
                has_foundry_label,
                task_key,
            }
        })
        .collect()
}

#[cfg(all(test, feature = "linear_backend"))]
mod tests {
    use super::build_existing_from_tuples;
    use crate::linear_executor::execute_plan;
    use crate::linear_phase_d::plan_from_markdown_and_existing;
    use crate::linear_reconcile::normalize_task_key;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn mk_tuple(
        id: &str,
        title: &str,
        open: bool,
        labeled: bool,
        task_key: Option<&str>,
    ) -> (String, String, bool, bool, Option<String>) {
        (
            id.to_string(),
            title.to_string(),
            open,
            labeled,
            task_key.map(|s| s.to_string()),
        )
    }

    #[test]
    fn builds_existing_with_keys_and_labels() {
        let rows = vec![
            mk_tuple("I1", "Add login flow", true, true, Some("add-login-flow")),
            mk_tuple("I2", "Old task", true, false, None),
        ];
        let existing = build_existing_from_tuples(rows);
        assert_eq!(existing.len(), 2);
        assert_eq!(existing[0].id, "I1");
        assert_eq!(existing[0].task_key.as_deref(), Some("add-login-flow"));
        assert!(existing[0].has_foundry_label);
        assert_eq!(existing[1].id, "I2");
        assert_eq!(existing[1].task_key.as_deref(), None);
        assert!(!existing[1].has_foundry_label);
    }

    #[test]
    fn normalized_title_matches_when_no_task_key() {
        let rows = vec![mk_tuple("I9", "Refactor: API / HTTP", true, true, None)];
        let existing = build_existing_from_tuples(rows);
        let normalized = normalize_task_key(&existing[0].title);
        assert_eq!(normalized, "refactor-api-http");
    }

    #[test]
    fn end_to_end_plan_and_execute_idempotent() {
        // Markdown desired tasks: keep one open, add one new, and leave one old to be closed
        let md = "- [ ] Keep me\n- [ ] New task\n";
        // Existing: one matching open, one extraneous open (to close), and one unlabeled match to trigger label fix
        let rows = vec![
            mk_tuple("E1", "Keep me ", true, true, Some("keep-me ")),
            mk_tuple("E2", "Old task", true, true, Some("old-task")),
            mk_tuple("E3", "Label me", true, false, Some("label-me")),
        ];
        let existing = build_existing_from_tuples(rows);

        let plan = plan_from_markdown_and_existing(md, &existing);

        // Record calls in order
        let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_c = Rc::clone(&calls);
        let calls_cl = Rc::clone(&calls);
        let calls_ro = Rc::clone(&calls);
        let calls_lb = Rc::clone(&calls);

        let mut create = move |t: &crate::linear_reconcile::DesiredTask| {
            calls_c.borrow_mut().push(format!("create:{}", t.text))
        };
        let mut close = move |id: &str| calls_cl.borrow_mut().push(format!("close:{}", id));
        let mut reopen = move |id: &str| calls_ro.borrow_mut().push(format!("reopen:{}", id));
        let mut label = move |id: &str| calls_lb.borrow_mut().push(format!("label:{}", id));

        execute_plan(&plan, &mut create, &mut close, &mut reopen, &mut label);

        // Second pass should produce identical sequence (idempotent behavior on caller side)
        let calls2: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let c2_c = Rc::clone(&calls2);
        let c2_cl = Rc::clone(&calls2);
        let c2_ro = Rc::clone(&calls2);
        let c2_lb = Rc::clone(&calls2);
        let mut create2 = move |t: &crate::linear_reconcile::DesiredTask| {
            c2_c.borrow_mut().push(format!("create:{}", t.text))
        };
        let mut close2 = move |id: &str| c2_cl.borrow_mut().push(format!("close:{}", id));
        let mut reopen2 = move |id: &str| c2_ro.borrow_mut().push(format!("reopen:{}", id));
        let mut label2 = move |id: &str| c2_lb.borrow_mut().push(format!("label:{}", id));
        execute_plan(&plan, &mut create2, &mut close2, &mut reopen2, &mut label2);

        assert_eq!(calls.borrow().clone(), calls2.borrow().clone());
    }
}
