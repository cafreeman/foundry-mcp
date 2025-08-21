//! Security utilities for path sanitization and input validation

use crate::errors::{ProjectManagerError, Result};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Security configuration for the project manager
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Base directory that all operations must be within
    pub base_directory: PathBuf,
    /// Maximum allowed path depth from base directory
    pub max_path_depth: usize,
    /// Maximum file size allowed (in bytes)
    pub max_file_size: u64,
    /// Maximum number of files that can be created
    pub max_file_count: usize,
    /// Whether to allow symbolic links
    pub allow_symlinks: bool,
    /// Allowed file extensions (if empty, all are allowed)
    pub allowed_extensions: Vec<String>,
    /// Blocked file extensions
    pub blocked_extensions: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            base_directory: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".foundry"),
            max_path_depth: 10,
            max_file_size: 50 * 1024 * 1024, // 50MB
            max_file_count: 10000,
            allow_symlinks: false,
            allowed_extensions: vec![
                "md".to_string(),
                "json".to_string(),
                "txt".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
            ],
            blocked_extensions: vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "com".to_string(),
                "pif".to_string(),
                "scr".to_string(),
                "vbs".to_string(),
                "js".to_string(),
                "jse".to_string(),
                "jar".to_string(),
                "sh".to_string(),
                "bash".to_string(),
                "ps1".to_string(),
                "php".to_string(),
                "py".to_string(),
                "rb".to_string(),
                "pl".to_string(),
            ],
        }
    }
}

/// Path sanitizer for security-critical operations
pub struct PathSanitizer {
    config: SecurityConfig,
}

impl PathSanitizer {
    /// Create a new path sanitizer with the given configuration
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Create a path sanitizer with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SecurityConfig::default())
    }

    /// Sanitize and validate a path relative to the base directory
    pub fn sanitize_path(&self, path: &str) -> Result<PathBuf> {
        // First, basic input validation
        self.validate_path_string(path)?;

        // Parse the path
        let path = Path::new(path);

        // Resolve the path relative to the base directory
        let full_path = self.config.base_directory.join(path);

        // Canonicalize to resolve any relative components
        let canonical_path = self.canonicalize_safely(&full_path)?;

        // Ensure the path is within the base directory
        self.ensure_within_base_directory(&canonical_path)?;

        // Check path depth
        self.check_path_depth(&canonical_path)?;

        // Validate file extension
        self.validate_file_extension(&canonical_path)?;

        Ok(canonical_path)
    }

    /// Sanitize a file name (no directory components)
    pub fn sanitize_filename(&self, filename: &str) -> Result<String> {
        // Check for dangerous characters
        if filename.contains('/') || filename.contains('\\') || filename.contains('\0') {
            return Err(ProjectManagerError::validation_error(
                "filename",
                filename,
                "Filename contains path separators or null characters",
            ));
        }

        // Check for reserved names on Windows
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        let filename_upper = filename.to_uppercase();
        if reserved_names.iter().any(|&name| {
            filename_upper == name || filename_upper.starts_with(&format!("{}.", name))
        }) {
            return Err(ProjectManagerError::validation_error(
                "filename",
                filename,
                "Filename is a reserved system name",
            ));
        }

        // Check for dangerous patterns
        if filename.starts_with('.') && filename.len() > 1 {
            // Allow some common dotfiles but block dangerous ones
            let allowed_dotfiles = [".gitignore", ".env.example", ".editorconfig"];
            if !allowed_dotfiles.contains(&filename) {
                return Err(ProjectManagerError::validation_error(
                    "filename",
                    filename,
                    "Hidden files are not allowed",
                ));
            }
        }

        // Check length
        if filename.len() > 255 {
            return Err(ProjectManagerError::validation_error(
                "filename",
                filename,
                "Filename too long (max 255 characters)",
            ));
        }

        // Validate extension
        if let Some(extension) = Path::new(filename).extension().and_then(OsStr::to_str) {
            if self
                .config
                .blocked_extensions
                .contains(&extension.to_lowercase())
            {
                return Err(ProjectManagerError::validation_error(
                    "filename",
                    filename,
                    &format!("File extension '{}' is not allowed", extension),
                ));
            }

            if !self.config.allowed_extensions.is_empty()
                && !self
                    .config
                    .allowed_extensions
                    .contains(&extension.to_lowercase())
            {
                return Err(ProjectManagerError::validation_error(
                    "filename",
                    filename,
                    &format!("File extension '{}' is not in the allowed list", extension),
                ));
            }
        }

        Ok(filename.to_string())
    }

    /// Validate a content string for security issues
    pub fn validate_content(&self, content: &str) -> Result<()> {
        // Check content length
        if content.len() > self.config.max_file_size as usize {
            return Err(ProjectManagerError::validation_error(
                "content",
                "content",
                &format!(
                    "Content exceeds maximum size of {} bytes",
                    self.config.max_file_size
                ),
            ));
        }

        // Check for binary content (null bytes)
        if content.contains('\0') {
            return Err(ProjectManagerError::validation_error(
                "content",
                "content",
                "Content contains binary data (null bytes)",
            ));
        }

        // Check for extremely long lines (potential DoS)
        const MAX_LINE_LENGTH: usize = 10000;
        for (line_num, line) in content.lines().enumerate() {
            if line.len() > MAX_LINE_LENGTH {
                return Err(ProjectManagerError::validation_error(
                    "content",
                    "content",
                    &format!(
                        "Line {} exceeds maximum length of {} characters",
                        line_num + 1,
                        MAX_LINE_LENGTH
                    ),
                ));
            }
        }

        // Check for suspicious patterns that might indicate code injection
        let suspicious_patterns = [
            "<?php",
            "<%",
            "%>",
            "<?=",
            "<script",
            "</script>",
            "javascript:",
            "data:text/html",
            "data:application/",
            "eval(",
            "exec(",
            "system(",
            "file_get_contents",
            "file_put_contents",
            "__import__",
            "import os",
            "import sys",
            "require(",
            "include(",
        ];

        let content_lower = content.to_lowercase();
        for pattern in &suspicious_patterns {
            if content_lower.contains(pattern) {
                return Err(ProjectManagerError::validation_error(
                    "content",
                    "content",
                    &format!("Content contains suspicious pattern: {}", pattern),
                ));
            }
        }

        Ok(())
    }

    /// Check if a path exists and is within our security boundaries
    pub fn validate_existing_path(&self, path: &Path) -> Result<()> {
        // Check if path exists
        if !path.exists() {
            return Ok(()); // Non-existent paths are fine for creation
        }

        // Check if it's a symlink when not allowed
        if !self.config.allow_symlinks && path.is_symlink() {
            return Err(ProjectManagerError::permission_error(
                "access",
                path,
                "Symbolic links are not allowed",
            ));
        }

        // Check file size if it's a file
        if path.is_file()
            && let Ok(metadata) = path.metadata()
                && metadata.len() > self.config.max_file_size {
                    return Err(ProjectManagerError::validation_error(
                        "file_size",
                        &path.display().to_string(),
                        &format!(
                            "File size exceeds maximum of {} bytes",
                            self.config.max_file_size
                        ),
                    ));
                }

        Ok(())
    }

    /// Validate path string for basic security issues
    fn validate_path_string(&self, path: &str) -> Result<()> {
        // Check for null bytes
        if path.contains('\0') {
            return Err(ProjectManagerError::validation_error(
                "path",
                path,
                "Path contains null bytes",
            ));
        }

        // Check for dangerous patterns
        if path.contains("..") {
            return Err(ProjectManagerError::validation_error(
                "path",
                path,
                "Path contains directory traversal attempt (..)",
            ));
        }

        // Check for absolute paths (shouldn't be used in our context)
        if path.starts_with('/') || (path.len() > 1 && path.chars().nth(1) == Some(':')) {
            return Err(ProjectManagerError::validation_error(
                "path",
                path,
                "Absolute paths are not allowed",
            ));
        }

        // Check for very long paths
        if path.len() > 1000 {
            return Err(ProjectManagerError::validation_error(
                "path",
                path,
                "Path is too long (max 1000 characters)",
            ));
        }

        Ok(())
    }

    /// Safely canonicalize a path without following symlinks
    fn canonicalize_safely(&self, path: &Path) -> Result<PathBuf> {
        let mut result = PathBuf::new();

        // Start with the base directory
        for component in self.config.base_directory.components() {
            result.push(component);
        }

        // Process the relative path components
        let relative_path = path
            .strip_prefix(&self.config.base_directory)
            .unwrap_or(path);

        for component in relative_path.components() {
            match component {
                Component::Normal(name) => {
                    // Validate the component name
                    if let Some(name_str) = name.to_str() {
                        self.sanitize_filename(name_str)?;
                    }
                    result.push(name);
                }
                Component::CurDir => {
                    // Skip current directory references
                    continue;
                }
                Component::ParentDir => {
                    return Err(ProjectManagerError::validation_error(
                        "path",
                        &path.display().to_string(),
                        "Parent directory references (..) are not allowed",
                    ));
                }
                _ => {
                    return Err(ProjectManagerError::validation_error(
                        "path",
                        &path.display().to_string(),
                        "Invalid path component",
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Ensure the path is within the base directory
    fn ensure_within_base_directory(&self, path: &Path) -> Result<()> {
        if !path.starts_with(&self.config.base_directory) {
            return Err(ProjectManagerError::permission_error(
                "access",
                path,
                "Path is outside the allowed directory",
            ));
        }
        Ok(())
    }

    /// Check if the path depth is within limits
    fn check_path_depth(&self, path: &Path) -> Result<()> {
        let relative_path = path
            .strip_prefix(&self.config.base_directory)
            .map_err(|_| {
                ProjectManagerError::internal_error(
                    "path_depth_check",
                    "Failed to strip base directory prefix",
                    None,
                )
            })?;

        let depth = relative_path.components().count();
        if depth > self.config.max_path_depth {
            return Err(ProjectManagerError::validation_error(
                "path_depth",
                &path.display().to_string(),
                &format!(
                    "Path depth {} exceeds maximum allowed depth of {}",
                    depth, self.config.max_path_depth
                ),
            ));
        }

        Ok(())
    }

    /// Validate file extension
    fn validate_file_extension(&self, path: &Path) -> Result<()> {
        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
            let ext_lower = extension.to_lowercase();

            if self.config.blocked_extensions.contains(&ext_lower) {
                return Err(ProjectManagerError::validation_error(
                    "file_extension",
                    &path.display().to_string(),
                    &format!("File extension '{}' is blocked", extension),
                ));
            }

            if !self.config.allowed_extensions.is_empty()
                && !self.config.allowed_extensions.contains(&ext_lower)
            {
                return Err(ProjectManagerError::validation_error(
                    "file_extension",
                    &path.display().to_string(),
                    &format!("File extension '{}' is not in the allowed list", extension),
                ));
            }
        }

        Ok(())
    }
}

/// Rate limiter for preventing abuse
pub struct RateLimiter {
    requests: std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>,
    max_requests: u32,
    window_duration: std::time::Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_duration: std::time::Duration) -> Self {
        Self {
            requests: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_requests,
            window_duration,
        }
    }

    /// Check if a request from the given identifier is allowed
    pub fn is_allowed(&self, identifier: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = std::time::Instant::now();

        // Clean up old entries
        requests.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.window_duration);

        // Check current identifier
        match requests.get_mut(identifier) {
            Some((count, timestamp)) => {
                if now.duration_since(*timestamp) >= self.window_duration {
                    // Reset window
                    *count = 1;
                    *timestamp = now;
                    true
                } else if *count < self.max_requests {
                    // Increment count
                    *count += 1;
                    true
                } else {
                    // Rate limit exceeded
                    false
                }
            }
            None => {
                // First request from this identifier
                requests.insert(identifier.to_string(), (1, now));
                true
            }
        }
    }

    /// Get the current request count for an identifier
    pub fn get_current_count(&self, identifier: &str) -> u32 {
        let requests = self.requests.lock().unwrap();
        let now = std::time::Instant::now();

        if let Some((count, timestamp)) = requests.get(identifier) {
            if now.duration_since(*timestamp) < self.window_duration {
                *count
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_sanitizer() -> (PathSanitizer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SecurityConfig::default();
        config.base_directory = temp_dir.path().to_path_buf();
        let sanitizer = PathSanitizer::new(config);
        (sanitizer, temp_dir)
    }

    #[test]
    fn test_sanitize_safe_path() {
        let (sanitizer, _temp_dir) = create_test_sanitizer();

        let result = sanitizer.sanitize_path("project1/spec.md");
        assert!(result.is_ok());

        let sanitized = result.unwrap();
        assert!(sanitized.ends_with("project1/spec.md"));
    }

    #[test]
    fn test_reject_directory_traversal() {
        let (sanitizer, _temp_dir) = create_test_sanitizer();

        let result = sanitizer.sanitize_path("../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_absolute_path() {
        let (sanitizer, _temp_dir) = create_test_sanitizer();

        let result = sanitizer.sanitize_path("/etc/passwd");
        assert!(result.is_err());

        let result = sanitizer.sanitize_path("C:\\Windows\\System32");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        let (sanitizer, _temp_dir) = create_test_sanitizer();

        assert!(sanitizer.sanitize_filename("valid-file.md").is_ok());
        assert!(sanitizer.sanitize_filename("file/path.md").is_err());
        assert!(sanitizer.sanitize_filename("CON.txt").is_err());
        assert!(sanitizer.sanitize_filename("file.exe").is_err());
    }

    #[test]
    fn test_validate_content() {
        let (sanitizer, _temp_dir) = create_test_sanitizer();

        assert!(
            sanitizer
                .validate_content("# Valid Markdown\n\nThis is safe content.")
                .is_ok()
        );
        assert!(
            sanitizer
                .validate_content("<?php echo 'dangerous'; ?>")
                .is_err()
        );
        assert!(
            sanitizer
                .validate_content("Content with\0null byte")
                .is_err()
        );
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, std::time::Duration::from_secs(60));

        assert!(limiter.is_allowed("user1"));
        assert!(limiter.is_allowed("user1"));
        assert!(!limiter.is_allowed("user1")); // Third request should be blocked

        assert!(limiter.is_allowed("user2")); // Different user should be allowed
    }
}
