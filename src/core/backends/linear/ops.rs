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
    let filter = ProjectFilterById { id: Some(IdEqComparator { eq: Some(cynic::Id::from(project_id.to_string())) }) };
    let pdq = ProjectDocumentsQuery::builder()
        .variables(ProjectDocumentsVars { filter: Some(filter), first: Some(1), docs_first: Some(50) })
        .build();
    let data = gql.execute(pdq).await?;

    let mut existing_vision: Option<DocumentLite> = None;
    let mut existing_tech: Option<DocumentLite> = None;

    if let Some(project) = data.projects_conn.nodes.into_iter().next() {
        for d in project.documents_conn.nodes.into_iter() {
            if d.title == "Vision" || d.title == format!("{} — Vision", project_name) {
                existing_vision = Some(d);
            } else if d.title == "Tech Stack" || d.title == format!("{} — Tech Stack", project_name) {
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
                    .variables(UpdateDocumentVars { id: doc.id.to_string(), input: DocumentUpdateInputLinear { title: None, content: Some(vision_body.clone()) } })
                    .build(),
            )
            .await?;
        doc.id.to_string()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::builder()
                    .variables(CreateDocumentVars { input: DocumentCreateInputLinear { title: format!("{} — Vision", project_name), content: Some(vision_body.clone()), projectId: Some(project_id.to_string()) } })
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
                    .variables(UpdateDocumentVars { id: doc.id.to_string(), input: DocumentUpdateInputLinear { title: None, content: Some(tech_body.clone()) } })
                    .build(),
            )
            .await?;
        doc.id.to_string()
    } else {
        let created = gql
            .execute(
                CreateDocumentOp::builder()
                    .variables(CreateDocumentVars { input: DocumentCreateInputLinear { title: format!("{} — Tech Stack", project_name), content: Some(tech_body.clone()), projectId: Some(project_id.to_string()) } })
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