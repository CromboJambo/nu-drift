//! Nu Drift - Async Learning State Manager
//!
//! This is the async runtime for nu-drift. It wraps the pure sync types
//! behind an `Arc<Mutex<>>` to handle concurrent access safely while
//! maintaining the immutable update semantics of the core system.

use std::sync::{Arc, Mutex};

mod types;
mod update;

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

/// Run multiple interactions and check for stuck concepts (async demo)
async fn run_demo(state: &State) {
    use types::{ConceptId, InteractionKind};

    println!("=== Async Learning Demo ===\n");

    let concept1 = ConceptId("async_await".to_string());

    println!("Recording initial interactions...");

    // First, apply the concept to create it in the state
    state
        .record_interaction(InteractionKind::Applied, &[concept1.clone()])
        .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Then simulate repeated asking (no progress, loop counter increases)
    for _ in 0..8 {
        state
            .record_interaction(InteractionKind::Asked, &[concept1.clone()])
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Small delay between calls
    }

    println!("\nChecking for stuck concepts...");
    let stuck = state.get_stuck_concepts().await;
    if stuck.is_empty() {
        println!("No concepts are stuck yet (loop detection needs more iterations)");
    } else {
        println!("Stuck concepts:");
        for (concept_id, loop_count) in stuck {
            println!("  - {}: {} loops", concept_id.0, loop_count);
        }
    }

    // Show current state as JSON
    let json = state.to_json().await.unwrap();
    println!("\nCurrent state:");
    println!("{}", json);
}

/// Main async entry point with tokio runtime
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_demo(&State::new()).await;
    Ok(())
}
