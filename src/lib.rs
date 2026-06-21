//! Nu Drift - Learning State Library
//!
//! Async-safe state wrapper around the pure update function.

use std::sync::{Arc, Mutex};

pub mod types;
pub mod types_display;
pub mod update;

use types::{ConceptId, InteractionKind, UserState};
use update::update;

/// Async-safe state wrapper using Arc<Mutex<>> pattern
/// This is the load-bearing ownership model for async code
#[derive(Clone)]
pub struct State {
    inner: Arc<Mutex<UserState>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UserState::default())),
        }
    }

    /// Get a snapshot of current state (clones the mutex contents)
    pub async fn get_state(&self) -> UserState {
        self.inner.lock().unwrap().clone()
    }

    /// Record an interaction and update beliefs atomically
    pub async fn record_interaction(
        &self,
        kind: InteractionKind,
        concepts_touched: &[ConceptId],
    ) -> types::InteractionId {
        let mut state = self.inner.lock().unwrap();
        let id = types::InteractionId(state.trajectory.len() as u64);
        let interaction = types::Interaction::new_from_trajectory(id.0, kind, concepts_touched);

        let old_state = std::mem::take(&mut *state);
        let new_state = update(old_state, interaction);
        *state = new_state;

        id
    }

    /// Update beliefs using the pure update function (preferred pattern)
    pub async fn apply_update(
        &self,
        interaction: update::Interaction,
    ) -> Result<types::UserState, String> {
        // Run pure update in-place with a single lock
        let mut state = self.inner.lock().unwrap();
        let old_state = std::mem::take(&mut *state);
        let new_state = update(old_state, interaction);
        *state = new_state.clone();
        Ok(new_state)
    }

    /// Set basecamp snapshot (atomic operation)
    pub async fn set_basecamp(&self, description: &str, threshold: f32) -> bool {
        let mut state = self.inner.lock().unwrap();
        state.set_basecamp(description, threshold)
    }

    /// Get stuck concepts for intervention queue (async-safe query)
    pub async fn get_stuck_concepts(&self) -> Vec<(types::ConceptId, u32)> {
        let state = self.inner.lock().unwrap();
        // Belief is owned in the HashMap, so we can collect owned values
        state
            .concepts
            .iter()
            .filter(|(_, b)| b.is_stuck())
            .map(|(k, b)| (k.clone(), b.loop_count))
            .collect()
    }

    /// Apply decay to all beliefs (time-based cleanup)
    pub async fn apply_decay(&self) {
        let mut state = self.inner.lock().unwrap();
        state.apply_all_decay();
    }

    /// Get concepts needing revisiting below threshold
    pub async fn needs_revisiting(&self, threshold: f32) -> Vec<(types::ConceptId, types::Belief)> {
        let state = self.inner.lock().unwrap();
        // Return owned values to avoid lifetime issues with mutex-locked data
        state
            .concepts
            .iter()
            .filter(|(_, b)| b.confidence < threshold)
            .map(|(k, b)| (k.clone(), b.clone()))
            .collect()
    }

    /// Serialize current state to JSON (async-safe query)
    pub async fn to_json(&self) -> Result<String, serde_json::Error> {
        let state = self.inner.lock().unwrap();
        state.to_json()
    }

    /// Deserialize and replace entire state from JSON
    pub async fn from_json(&self, json: &str) -> Result<(), serde_json::Error> {
        let new_state = types::UserState::from_json(json)?;
        *self.inner.lock().unwrap() = new_state;
        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
