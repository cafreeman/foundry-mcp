//! Data access layer for projects and specifications

pub mod project;
pub mod specification;

pub use project::ProjectRepository;
pub use specification::SpecificationRepository;
