use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        }
    }

    /// Apply time-based decay to confidence
    pub fn apply_decay(&mut self, hours_since_last_seen: f32) {
        let decay_amount = self.decay_rate * hours_since_last_seen;
        self.confidence = (self.confidence - decay_amount).max(0.0);
        self.last_seen = Utc::now();
    }

    /// Update confidence with new evidence
    pub fn update_confidence(&mut self, new_confidence: f32) {
        assert!(new_confidence >= 0.0 && new_confidence <= 1.0);
        self.confidence = new_confidence;
        self.last_seen = Utc::now();
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

    /// Record an interaction and update related beliefs (mutable API)
    pub fn record_interaction(
        &mut self,
        kind: InteractionKind,
        concepts_touched: &[ConceptId],
    ) -> InteractionId {
        let id = InteractionId(self.trajectory.len() as u64);
        let mut interaction = Interaction::new_from_trajectory(id.0, kind, concepts_touched);

        // Update beliefs based on interaction type
        for concept_id in &interaction.concepts_touched {
            if let Some(belief) = self.concepts.get_mut(concept_id) {
                belief.add_context(id);

                match kind {
                    InteractionKind::Applied => {
                        // Application should increase confidence
                        belief.update_confidence((belief.confidence + 0.2).min(1.0));
                        interaction.resolved = true;
                    }
                    InteractionKind::Asked | InteractionKind::Confused => {
                        // Questions don't directly change confidence, just add context
                    }
                }
            } else if kind == InteractionKind::Applied {
                // New concept that was applied - start with higher baseline
                self.concepts
                    .insert(concept_id.clone(), Belief::new(0.6, 0.15));
            }
        }

        self.trajectory.push(interaction);
        id
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
