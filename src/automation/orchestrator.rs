use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::types::{ConceptId, InteractionKind};
use crate::State;
use super::auto_tracker::AutoTracker;
use super::concept_extractor::extract_concepts;

/// Agent orchestrator for automatic concept tracking
pub struct AgentOrchestrator {
    state: Arc<Mutex<State>>,
    tracker: AutoTracker,
    system_prompt: Arc<RwLock<String>>,
}

impl AgentOrchestrator {
    /// Create a new agent orchestrator
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self {
            state,
            tracker: AutoTracker::new(state.clone()),
            system_prompt: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Run a user interaction and automatically track concepts
    pub async fn run_interaction(&self, user_message: &str) -> String {
        // Get stuck concepts to add to context
        let stuck = self.tracker.get_stuck_concepts().await;
        if !stuck.is_empty() {
            self.add_stuck_context(&stuck);
        }

        // Get concepts needing revisiting
        let needs_revisit = self.tracker.needs_revisiting(0.5).await;
        if !needs_revisit.is_empty() {
            self.add_revisit_context(&needs_revisit);
        }

        // Create interaction from user message
        let interaction = crate::types::Interaction {
            id: crate::types::InteractionId(0),
            kind: InteractionKind::Asked,
            concepts_touched: extract_concepts(user_message, InteractionKind::Asked),
            resolved: false,
            at: chrono::Utc::now(),
        };

        // Record interaction in state
        self.tracker.track_interaction(interaction).await;

        // Get response from agent (placeholder)
        let response = self.get_agent_response(user_message).await;

        // Create interaction from agent response
        let response_interaction = crate::types::Interaction {
            id: crate::types::InteractionId(1),
            kind: InteractionKind::Applied,
            concepts_touched: extract_concepts(&response, InteractionKind::Applied),
            resolved: true,
            at: chrono::Utc::now(),
        };

        // Record agent response in state
        self.tracker.track_interaction(response_interaction).await;

        response
    }

    /// Add stuck concepts to system prompt
    fn add_stuck_context(&self, stuck: Vec<(ConceptId, u32)>) {
        let stuck_str = stuck.iter()
            .map(|(id, count)| format!("{} ({} loops)", id.0, count))
            .collect::<Vec<_>>()
            .join(", ");

        let mut prompt = self.system_prompt.write().await;
        prompt.push_str(&format!("\n[Stuck Concepts]: {}\n", stuck_str));
    }

    /// Add concepts needing revisiting to system prompt
    fn add_revisit_context(&self, needs_revisit: Vec<(ConceptId, crate::types::Belief)>) {
        let revisit_str = needs_revisit.iter()
            .map(|(id, belief)| format!("{} (confidence: {:.2})", id.0, belief.confidence))
            .collect::<Vec<_>>()
            .join(", ");

        let mut prompt = self.system_prompt.write().await;
        prompt.push_str(&format!("\n[Needs Revisiting]: {}\n", revisit_str));
    }

    /// Get agent response (placeholder for actual agent integration)
    async fn get_agent_response(&self, user_message: &str) -> String {
        // In production, this would call the actual agent
        format!("I understand: {}", user_message)
    }

    /// Get current system prompt
    pub async fn get_system_prompt(&self) -> String {
        self.system_prompt.read().await.clone()
    }

    /// Get current state
    pub async fn get_state(&self) -> crate::types::UserState {
        self.tracker.get_state().await
    }

    /// Get stuck concepts
    pub async fn get_stuck_concepts(&self) -> Vec<(ConceptId, u32)> {
        self.tracker.get_stuck_concepts().await
    }

    /// Get concepts needing revisiting
    pub async fn needs_revisiting(&self, threshold: f32) -> Vec<(ConceptId, crate::types::Belief)> {
        self.tracker.needs_revisiting(threshold).await
    }

    /// Get last applied interactions
    pub async fn last_applied(&self, n: usize) -> Vec<crate::types::Interaction> {
        self.tracker.last_applied(n).await
    }

    /// Get basecamp status
    pub async fn check_basecamp(&self) -> Option<crate::types::Snapshot> {
        self.tracker.check_basecamp().await
    }
}
