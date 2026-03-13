use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tolerance for considering confidence "not moving" - tuned to avoid false positives
const EPSILON: f32 = 0.01;

/// Unique identifier for a concept being tracked
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub String);

impl From<&str> for ConceptId {
    fn from(s: &str) -> Self {
        ConceptId(s.to_string())
    }
}

/// Unique identifier for an interaction record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InteractionId(pub u64);

impl Default for InteractionId {
    fn default() -> Self {
        InteractionId(0)
    }
}

/// The agent's model of what the user understands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    /// Confidence level from 0.0 to 1.0
    /// This is calibrated uncertainty, not a binary flag.
    /// A value of 0.7 means "I think they're ~70% solid on this"
    pub confidence: f32,

    /// When we last observed evidence for this belief
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_seen: DateTime<Utc>,

    /// Context chain - proof by implication
    /// References to interactions that support this belief
    pub context: Vec<InteractionId>,

    /// How quickly this knowledge decays when untouched
    /// Higher = faster decay, lower = more stable
    #[serde(rename = "decay_rate")]
    pub decay_rate: f32,

    /// Number of consecutive iterations where confidence hasn't moved meaningfully
    /// This detects loops: agent keeps working on same concept without progress
    #[serde(default)]
    pub loop_count: u32,

    /// Confidence change from last update - trending up or down?
    /// Used to detect if we're stuck in a pattern of no progress
    #[serde(default)]
    pub loop_delta: f32,

    /// Previous confidence value for delta calculation (not serialized)
    #[serde(skip)]
    pub last_confidence: Option<f32>,
}

impl Belief {
    pub fn new(confidence: f32, decay_rate: f32) -> Self {
        assert!(confidence >= 0.0 && confidence <= 1.0);
        assert!(decay_rate >= 0.0);

        Self {
            confidence,
            last_seen: Utc::now(),
            context: Vec::new(),
            decay_rate,
            loop_count: 0,
            loop_delta: 0.0,
            last_confidence: None,
        }
    }

    /// Update confidence and track loop detection metrics
    pub fn update_confidence_with_loop_tracking(&mut self, new_confidence: f32) {
        assert!(new_confidence >= 0.0 && new_confidence <= 1.0);

        // Calculate delta from previous confidence
        let delta = new_confidence - self.confidence;
        self.loop_delta = delta;
        self.last_confidence = Some(self.confidence);

        if delta.abs() < EPSILON {
            // Not moving meaningfully — potential loop
            self.loop_count += 1;
        } else if delta > 0.0 {
            // Making progress — reset loop counter
            self.loop_count = 0;
        } else {
            // Moving but wrong direction — still a loop
            self.loop_count += 1;
        }

        self.confidence = new_confidence;
        self.last_seen = Utc::now();
    }

    /// Reset loop counter when making progress on a concept
    pub fn reset_loop_count(&mut self) {
        self.loop_count = 0;
        self.loop_delta = 0.0;
        self.last_confidence = None;
    }

    /// Check if this belief is stuck (high loop count with no positive progress)
    pub fn is_stuck(&self) -> bool {
        const LOOP_THRESHOLD: u32 = 5; // Tunable parameter
        self.loop_count > LOOP_THRESHOLD && self.loop_delta <= 0.0
    }

    /// Apply time-based decay to confidence
    pub fn apply_decay(&mut self, hours_since_last_seen: f32) {
        let decay_amount = self.decay_rate * hours_since_last_seen;
        self.confidence = (self.confidence - decay_amount).max(0.0);
        self.last_seen = Utc::now();
    }

    /// Update confidence with new evidence (legacy method, uses loop tracking)
    #[deprecated(
        since = "0.2.0",
        note = "Use update_confidence_with_loop_tracking instead"
    )]
    pub fn update_confidence(&mut self, new_confidence: f32) {
        assert!(new_confidence >= 0.0 && new_confidence <= 1.0);
        self.update_confidence_with_loop_tracking(new_confidence);
    }

    /// Check if this belief needs revisiting (confidence below threshold)
    pub fn needs_revisiting(&self, threshold: f32) -> bool {
        self.confidence < threshold
    }

    /// Get concepts where agent is stuck - high loop count with no positive progress
    pub fn get_stuck_concepts(concepts: &HashMap<ConceptId, Belief>) -> Vec<(ConceptId, u32)> {
        let mut stuck: Vec<_> = concepts
            .iter()
            .filter(|(_, b)| b.is_stuck())
            .map(|(k, b)| (k.clone(), b.loop_count))
            .collect();

        // Sort by loop count descending - most stuck first
        stuck.sort_by(|a, b| b.1.cmp(&a.1));
        stuck
    }

    /// Add context reference (proof by implication)
    pub fn add_context(&mut self, interaction_id: InteractionId) {
        if !self.context.contains(&interaction_id) {
            self.context.push(interaction_id);
        }
    }

    /// Add context proof for pure update function usage
    #[allow(dead_code)]
    pub fn add_context_proof(&mut self, id: InteractionId) {
        self.add_context(id);
    }
}

/// What kind of learning event occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionKind {
    /// User asked a question about the concept
    Asked,

    /// User expressed confusion or uncertainty
    Confused,

    /// User applied knowledge in practice
    Applied,

    /// Agent has detected stuck pattern - confidence not moving after repeated attempts
    Stuck,
}

/// A single learning interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: InteractionId,
    pub kind: InteractionKind,
    pub concepts_touched: Vec<ConceptId>,
    pub resolved: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub at: DateTime<Utc>,
}

impl Interaction {
    pub fn new(id: InteractionId, kind: InteractionKind, concepts_touched: &[ConceptId]) -> Self {
        Self {
            id,
            kind,
            concepts_touched: concepts_touched.to_vec(),
            resolved: false,
            at: Utc::now(),
        }
    }

    /// Create interaction with auto-incremented ID from trajectory length
    pub fn new_from_trajectory(
        id: u64,
        kind: InteractionKind,
        concepts_touched: &[ConceptId],
    ) -> Self {
        Self {
            id: InteractionId(id),
            kind,
            concepts_touched: concepts_touched.to_vec(),
            resolved: false,
            at: Utc::now(),
        }
    }
}

/// User-authored snapshot marking a stable point in their learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub description: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub snapshot_at: DateTime<Utc>,
    /// Minimum confidence across concepts at this point
    pub confidence_threshold: f32,
}

impl Snapshot {
    pub fn new(description: &str, confidence_threshold: f32) -> Self {
        assert!(confidence_threshold >= 0.0 && confidence_threshold <= 1.0);

        Self {
            description: description.to_string(),
            snapshot_at: Utc::now(),
            confidence_threshold,
        }
    }
}

/// The complete user state - what the agent knows about what you know
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    /// Map of concept IDs to their current belief state
    pub concepts: HashMap<ConceptId, Belief>,

    /// Chronological record of all interactions
    pub trajectory: Vec<Interaction>,

    /// Optional user-authored stable point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basecamp: Option<Snapshot>,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            concepts: HashMap::new(),
            trajectory: Vec::new(),
            basecamp: None,
        }
    }
}

impl UserState {
    /// Get or create belief for a concept (starts at 0.5 confidence)
    pub fn get_or_create_belief(&mut self, concept_id: ConceptId) -> &mut Belief {
        self.concepts
            .entry(concept_id)
            .or_insert_with(|| Belief::new(0.5, 0.1))
    }

    /// Set a basecamp snapshot at current state
    pub fn set_basecamp(&mut self, description: &str, confidence_threshold: f32) -> bool {
        let min_confidence = self
            .concepts
            .values()
            .map(|b| b.confidence)
            .fold(f32::MAX, |a, b| a.min(b));

        if min_confidence >= confidence_threshold && !self.concepts.is_empty() {
            self.basecamp = Some(Snapshot::new(description, min_confidence));
            true
        } else {
            eprintln!(
                "Cannot set basecamp: minimum confidence {:.2} < threshold {:.2}",
                min_confidence, confidence_threshold
            );
            false
        }
    }

    /// Apply decay to all beliefs based on time elapsed
    pub fn apply_all_decay(&mut self) {
        let now = Utc::now();

        for belief in self.concepts.values_mut() {
            let hours_elapsed = (now - belief.last_seen).num_seconds() as f32 / 3600.0;
            if hours_elapsed > 0.0 {
                belief.apply_decay(hours_elapsed);
            }
        }
    }

    /// Get concepts needing revisiting (confidence below threshold)
    pub fn needs_revisiting(&self, threshold: f32) -> Vec<(&ConceptId, &Belief)> {
        self.concepts
            .iter()
            .filter(|(_, b)| b.confidence < threshold)
            .collect()
    }

    /// Get concepts where agent is stuck - high loop count with no positive progress
    pub fn get_stuck_concepts(&self) -> Vec<(ConceptId, u32)> {
        self.concepts
            .iter()
            .filter(|(_, b)| b.is_stuck())
            .map(|(k, b)| (k.clone(), b.loop_count))
            .collect()
    }

    /// Get last N applied interactions
    pub fn last_applied(&self, n: usize) -> Vec<&Interaction> {
        self.trajectory
            .iter()
            .filter(|i| i.kind == InteractionKind::Applied)
            .rev()
            .take(n)
            .collect()
    }

    /// Serialize state to JSON for persistence or tool use
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize state from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::{update, Interaction};

    #[test]
    fn test_loop_count_increases_on_asked_interactions() {
        let mut state = UserState::default();
        let concept = ConceptId("test".to_string());

        // Apply first to create the belief
        let interaction = Interaction::new_from_trajectory(
            state.trajectory.len() as u64,
            InteractionKind::Applied,
            &[concept.clone()],
        );
        state = update(state, interaction);

        // Multiple Asked interactions should increase loop count
        for _ in 0..10 {
            let interaction = Interaction::new_from_trajectory(
                state.trajectory.len() as u64,
                InteractionKind::Asked,
                &[concept.clone()],
            );
            state = update(state, interaction);
        }

        let belief = state.concepts.get(&concept).unwrap();
        assert!(
            belief.loop_count >= 8,
            "Loop count should be at least 8 after 10 Asked interactions"
        );
    }

    #[test]
    fn test_stuck_detection_triggers_after_threshold() {
        let mut state = UserState::default();
        let concept = ConceptId("stuck_concept".to_string());

        // Apply first to create the belief with high confidence
        let interaction = Interaction::new_from_trajectory(
            state.trajectory.len() as u64,
            InteractionKind::Applied,
            &[concept.clone()],
        );
        state = update(state, interaction);

        // Manually set up a stuck state (high loop count, non-positive delta)
        if let Some(belief) = state.concepts.get_mut(&concept) {
            belief.loop_count = 6;
            belief.loop_delta = -0.01;
        }

        let stuck = state.get_stuck_concepts();
        assert_eq!(stuck.len(), 1, "Should detect the concept as stuck");
    }

    #[test]
    fn test_loop_reset_on_application() {
        let mut state = UserState::default();
        let concept = ConceptId("reset_test".to_string());

        // Apply first to create belief
        let interaction = Interaction::new_from_trajectory(
            state.trajectory.len() as u64,
            InteractionKind::Applied,
            &[concept.clone()],
        );
        state = update(state, interaction);

        // Some Asked interactions (increases loop count)
        for _ in 0..5 {
            let interaction = Interaction::new_from_trajectory(
                state.trajectory.len() as u64,
                InteractionKind::Asked,
                &[concept.clone()],
            );
            state = update(state, interaction);
        }

        // Another Application should reset the loop counter
        let interaction = Interaction::new_from_trajectory(
            state.trajectory.len() as u64,
            InteractionKind::Applied,
            &[concept.clone()],
        );
        state = update(state, interaction);

        let belief = state.concepts.get(&concept).unwrap();
        assert!(
            belief.loop_count < 5,
            "Loop count should be reduced after application"
        );
    }
}
