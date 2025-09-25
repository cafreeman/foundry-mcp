use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditCommandTarget {
    Spec,
    Tasks,
    Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditCommandName {
    SetTaskStatus,
    UpsertTask,
    AppendToSection,
    RemoveListItem,
    RemoveFromSection,
    RemoveSection,
    ReplaceListItem,
    ReplaceInSection,
    ReplaceSectionContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditSelector {
    Section {
        value: String,
    },
    TaskText {
        value: String,
        #[serde(default)]
        section_context: Option<String>,
    },
    TextContent {
        value: String,
    },
    TextInSection {
        section: String,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Done,
    Todo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCommand {
    pub target: EditCommandTarget,
    pub command: EditCommandName,
    pub selector: EditSelector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpdateSummary {
    pub target: EditCommandTarget,
    pub applied: usize,
    pub skipped_idempotent: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorCandidate {
    pub selector_suggestion: EditSelector,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCommandError {
    pub target: EditCommandTarget,
    pub command_index: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates: Option<Vec<SelectorCandidate>>,
}
