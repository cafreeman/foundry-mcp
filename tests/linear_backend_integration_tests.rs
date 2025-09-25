//! Integration tests for the Linear backend using httpmock and isolated envs
//! These tests mirror the filesystem backend patterns but target Linear APIs.

mod common;

#[cfg(feature = "linear_backend")]
mod linear_integration {
    use crate::common::TestEnvironment;
    use foundry_mcp::core::backends::FoundryBackend;
    use foundry_mcp::core::backends::linear::{LinearBackend, LinearConfig};
    use foundry_mcp::types::project::ProjectConfig;
    use foundry_mcp::types::spec::{SpecConfig, SpecContentData};
    use httpmock::prelude::*;
    use serde_json::json;
    use std::time::Duration;
    use url::Url;

    fn cfg_for(server: &MockServer) -> LinearConfig {
        LinearConfig {
            endpoint: Url::parse(&server.url("/")).unwrap(),
            token: "test-token".to_string(),
            user_agent: "foundry-mcp-test".to_string(),
            timeout: Duration::from_secs(10),
            retry: foundry_mcp::core::backends::linear::config::RetryConfig::default(),
            team_id: None,
            team_key: Some("TEST".to_string()),
            team_name: None,
        }
    }

    #[test]
    fn test_linear_create_project_integration() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let server = MockServer::start();

            // 1) FindProjects → empty
            let _find_projects = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"FindProjects\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": { "projects": { "nodes": [] } }
                    }));
            });

            // 2) CreateProjectOp → returns project
            let _create_project = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateProjectOp\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "projectCreate": {
                                "project": {"id": "proj1", "name": "Test Project", "description": "desc"}
                            }
                        }
                    }));
            });

            // 3) UpdateProjectOp → ok
            let _update_project = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"UpdateProjectOp\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {"projectUpdate": {"project": {"id": "proj1", "name": "Test Project", "description": "updated"}}}
                    }));
            });

            // 4) ProjectDocumentsQuery → no docs
            let _project_docs = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"ProjectDocumentsQuery\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "projects": {
                                "nodes": [
                                    {"documents": {"nodes": []}}
                                ]
                            }
                        }
                    }));
            });

            // 5) CreateDocumentOp (Vision)
            let _doc_create_vision = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateDocumentOp\"")
                    .body_contains("Vision");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "documentCreate": {
                                "document": {"id": "doc-vision", "title": "Test Project — Vision", "url": "https://linear/doc-vision"}
                            }
                        }
                    }));
            });

            // 6) CreateDocumentOp (Tech Stack)
            let _doc_create_tech = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateDocumentOp\"")
                    .body_contains("Tech Stack");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "documentCreate": {
                                "document": {"id": "doc-tech", "title": "Test Project — Tech Stack", "url": "https://linear/doc-tech"}
                            }
                        }
                    }));
            });

            let cfg = cfg_for(&server);
            let backend = LinearBackend::new(&cfg).unwrap();

            let project_config = ProjectConfig {
                name: "Test Project".to_string(),
                vision: "Vision content".to_string(),
                tech_stack: "Tech content".to_string(),
                summary: "Summary".to_string(),
            };

            let project = backend.create_project(project_config).await.unwrap();
            assert_eq!(project.name, "Test Project");
            assert!(project.vision.as_deref().unwrap().contains("Vision"));
            assert!(project.tech_stack.as_deref().unwrap().contains("Tech"));
        });
    }

    #[test]
    fn test_linear_create_spec_integration() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let server = MockServer::start();

            // FindProjects → empty, then CreateProject
            let _find_projects = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"FindProjects\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({"data": {"projects": {"nodes": []}}}));
            });
            let _create_project = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateProjectOp\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {"projectCreate": {"project": {"id": "proj1", "name": "Test Project", "description": null}}}
                    }));
            });

            // Notes document creation
            let _doc_create_notes = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateDocumentOp\"")
                    .body_contains("Notes");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "documentCreate": {
                                "document": {"id": "doc-notes", "title": "User authentication — Notes", "url": "https://linear/doc-notes"}
                            }
                        }
                    }));
            });

            // Ensure foundry label (present)
            let _find_labels = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"FindIssueLabels\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {"issueLabels": {"nodes": [{"id": "label1", "name": "foundry"}]}}
                    }));
            });

            // Resolve team id by key
            let _find_teams = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"FindTeams\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {"teams": {"nodes": [{"id": "team1", "name": "Test Team", "key": "TEST"}]}}
                    }));
            });

            // Create issue for spec
            let _issue_create = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"CreateIssueOp\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "issueCreate": {
                                "issue": {"id": "issue1", "title": "User authentication", "description": "desc", "url": "https://linear/issue1"}
                            }
                        }
                    }));
            });

            let cfg = cfg_for(&server);
            let backend = LinearBackend::new(&cfg).unwrap();

            let spec_config = SpecConfig {
                project_name: "Test Project".to_string(),
                feature_name: "user_authentication".to_string(),
                content: SpecContentData {
                    spec: "Spec body".to_string(),
                    tasks: "- [ ] Task".to_string(),
                    notes: "Notes body".to_string(),
                },
            };

            let spec = backend.create_spec(spec_config).await.unwrap();
            assert_eq!(spec.project_name, "Test Project");
            assert!(spec.name.contains("user_authentication"));
            assert!(spec.location_hint.as_deref().unwrap().contains("linear"));
        });
    }

    #[test]
    fn test_linear_list_specs_integration() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let server = MockServer::start();

            // ListSpecIssuesQuery → one spec with marker
            let _list_specs = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"operationName\":\"ListSpecIssuesQuery\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(json!({
                        "data": {
                            "issues": {
                                "nodes": [
                                    {
                                        "id": "issue1",
                                        "title": "User authentication",
                                        "description": "<!-- foundry:specId=20240101_120000_user_authentication; type=spec; v=1 -->\nSpec content",
                                        "createdAt": "2024-01-01T00:00:00Z",
                                        "labels": {"nodes": [{"id": "label1", "name": "foundry"}]},
                                        "project": {"id": "proj1", "name": "Test Project", "description": null}
                                    }
                                ],
                                "pageInfo": {"hasNextPage": false, "endCursor": null}
                            }
                        }
                    }));
            });

            let cfg = cfg_for(&server);
            let backend = LinearBackend::new(&cfg).unwrap();

            let specs = backend.list_specs("Test Project").await.unwrap();
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].project_name, "Test Project");
            assert!(specs[0].name.contains("user_authentication"));
        });
    }
}

