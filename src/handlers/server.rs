//! MCP server handler implementation

/// Main MCP server handler for Project Manager
pub struct ProjectManagerHandler;

impl ProjectManagerHandler {
    /// Create a new ProjectManagerHandler instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectManagerHandler {
    fn default() -> Self {
        Self::new()
    }
}
