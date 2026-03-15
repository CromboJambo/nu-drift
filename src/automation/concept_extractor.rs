//! Automatic concept detection and tracking
//!
//! This module provides lightweight concept extraction from user interactions,
//! avoiding complex regex patterns that can cause infinite loops.

use crate::types::{ConceptId, InteractionKind};

/// Extract concepts from tool calls or user messages
///
/// Uses simple pattern matching instead of complex regex to avoid infinite loops.
/// Returns a vector of concept IDs that were detected.
pub fn extract_concepts(input: &str, kind: InteractionKind) -> Vec<ConceptId> {
    let mut concepts = Vec::new();
    let lower_input = input.to_lowercase();

    // Framework and language patterns
    if lower_input.contains("rust") {
        concepts.push(ConceptId("rust_programming".to_string()));
    }
    if lower_input.contains("typescript") || lower_input.contains("ts") {
        concepts.push(ConceptId("typescript".to_string()));
    }
    if lower_input.contains("python") {
        concepts.push(ConceptId("python_programming".to_string()));
    }
    if lower_input.contains("javascript") || lower_input.contains("js") {
        concepts.push(ConceptId("javascript".to_string()));
    }
    if lower_input.contains("docker") {
        concepts.push(ConceptId("docker".to_string()));
    }
    if lower_input.contains("kubernetes") {
        concepts.push(ConceptId("kubernetes".to_string()));
    }

    // Architecture and design patterns
    if lower_input.contains("api") {
        concepts.push(ConceptId("api_design".to_string()));
    }
    if lower_input.contains("database") {
        concepts.push(ConceptId("database_design".to_string()));
    }
    if lower_input.contains("microservice") {
        concepts.push(ConceptId("microservice_architecture".to_string()));
    }
    if lower_input.contains("event") {
        concepts.push(ConceptId("event_driven_architecture".to_string()));
    }

    // Development workflow concepts
    if lower_input.contains("cli") {
        concepts.push(ConceptId("cli_tools".to_string()));
    }
    if lower_input.contains("script") {
        concepts.push(ConceptId("automation_scripts".to_string()));
    }
    if lower_input.contains("test") {
        concepts.push(ConceptId("testing".to_string()));
    }
    if lower_input.contains("debug") {
        concepts.push(ConceptId("debugging".to_string()));
    }

    // Project management concepts
    if lower_input.contains("project") {
        concepts.push(ConceptId("project_management".to_string()));
    }
    if lower_input.contains("workflow") {
        concepts.push(ConceptId("workflow_optimization".to_string()));
    }

    // Specific to the nu-drift project itself
    if lower_input.contains("state") {
        concepts.push(ConceptId("state_management".to_string()));
    }
    if lower_input.contains("belief") {
        concepts.push(ConceptId("belief_tracking".to_string()));
    }
    if lower_input.contains("confidence") {
        concepts.push(ConceptId("confidence_estimation".to_string()));
    }
    if lower_input.contains("basecamp") {
        concepts.push(ConceptId("basecamp_concept".to_string()));
    }

    // Interaction-specific concepts
    if kind == InteractionKind::Applied {
        concepts.push(ConceptId("practical_application".to_string()));
    } else if kind == InteractionKind::Asked {
        concepts.push(ConceptId("information_retrieval".to_string()));
    } else if kind == InteractionKind::Confused {
        concepts.push(ConceptId("uncertainty_handling".to_string()));
    }

    // Remove duplicates while preserving order
    concepts.sort();
    concepts.dedup();

    concepts
}

/// Check if a concept already exists in a set
pub fn concept_exists(concepts: &[ConceptId], concept: &ConceptId) -> bool {
    concepts.iter().any(|c| c == concept)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_concepts() {
        let input = "I need to build a Rust program";
        let concepts = extract_concepts(input, InteractionKind::Applied);

        assert!(concepts.iter().any(|c| c.0.contains("rust")));
        assert!(concepts.len() >= 1);
    }

    #[test]
    fn test_extract_multiple_concepts() {
        let input = "Build a Docker container with Rust and TypeScript";
        let concepts = extract_concepts(input, InteractionKind::Applied);

        assert!(concepts.iter().any(|c| c.0.contains("rust")));
        assert!(concepts.iter().any(|c| c.0.contains("typescript")));
        assert!(concepts.iter().any(|c| c.0.contains("docker")));
        assert_eq!(concepts.len(), 3);
    }

    #[test]
    fn test_no_duplicates() {
        let input = "Rust rust rust";
        let concepts = extract_concepts(input, InteractionKind::Applied);

        assert_eq!(concepts.len(), 1);
    }

    #[test]
    fn test_empty_input() {
        let concepts = extract_concepts("", InteractionKind::Applied);
        assert!(concepts.is_empty());
    }

    #[test]
    fn test_interaction_specific_concepts() {
        let input = "I am confused about this";
        let concepts = extract_concepts(input, InteractionKind::Confused);

        assert!(concepts.iter().any(|c| c.0.contains("uncertainty")));
        assert!(concepts.iter().any(|c| c.0.contains("information")));
    }
}
