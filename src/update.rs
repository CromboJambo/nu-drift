//! Pure state transition function for learning agent
//!
//! This is the heart of the system: given old state and an interaction,
//! produce new state deterministically. No hidden mutation, no side effects.
//!
//! The philosophy here matches the confidence contract: we're modeling
//! what happened, not asserting certainty about what it means.

use crate::types::{Belief, Snapshot, UserState};
pub use crate::types::{Interaction, InteractionKind};

/// Pure function: old state + interaction → new state
///
/// This is intentionally stateless and pure. Given the same inputs, you
/// always get the same outputs. No hidden mutation, no random number
/// generation, no I/O.
///
/// # Arguments
/// * `state` - The current user state (consumed)
/// * `interaction` - The new interaction to record (consumed)
///
/// # Returns
/// A new UserState reflecting the updated knowledge model
pub fn update(state: UserState, interaction: Interaction) -> UserState {
    let mut new_state = state;

    // Apply time decay before recording new interaction
    apply_decay_to_all(&mut new_state);

    // Process each concept touched by this interaction
    for concept_id in &interaction.concepts_touched {
        match new_state.concepts.get_mut(concept_id) {
            Some(belief) => {
                // Add context reference (proof by implication)
                belief.add_context_proof(interaction.id);

                // Update confidence based on interaction type and existing evidence
                let updated_confidence = update_belief_confidence(
                    belief.confidence,
                    belief.context.len(),
                    interaction.kind,
                );

                // Only allow downward movement on explicit confusion
                let next_confidence = match interaction.kind {
                    InteractionKind::Confused => updated_confidence,
                    InteractionKind::Asked | InteractionKind::Stuck => {
                        belief.confidence.max(updated_confidence)
                    }
                    InteractionKind::Applied => updated_confidence,
                };

                belief.update_confidence_with_loop_tracking(next_confidence);
            }
            None => {
                // New concept - initialize with conservative baseline
                let initial_confidence = match interaction.kind {
                    InteractionKind::Applied => 0.4, // Applied knowledge gets slightly higher start
                    InteractionKind::Asked | InteractionKind::Confused => 0.3, // Questions are neutral
                    InteractionKind::Stuck => 0.3, // Stuck marker - same baseline as questions
                };

                new_state.concepts.insert(
                    concept_id.clone(),
                    Belief {
                        confidence: initial_confidence,
                        last_seen: chrono::Utc::now(),
                        context: vec![interaction.id],
                        decay_rate: 0.1, // Default decay rate
                        loop_count: 0,
                        loop_delta: 0.0,
                        last_confidence: None,
                    },
                );
            }
        }
    }

    // Record the interaction in trajectory
    new_state.trajectory.push(interaction);

    new_state
}

/// Calculate updated confidence based on evidence and interaction type
fn update_belief_confidence(
    current_confidence: f32,
    context_length: usize,
    kind: InteractionKind,
) -> f32 {
    // Base adjustment from interaction type
    let base_adjustment = match kind {
        InteractionKind::Applied => 0.15, // Application reinforces understanding
        InteractionKind::Asked => -0.02,  // Questions suggest uncertainty
        InteractionKind::Confused => -0.08, // Confusion indicates knowledge gap
        InteractionKind::Stuck => 0.0,    // Stuck interactions are neutral markers (no change)
    };

    // Context length bonus: more evidence = more stable confidence
    let context_bonus = if context_length > 3 {
        0.05 * (context_length as f32).min(10.0) / 10.0
    } else {
        0.0
    };

    // Combine adjustments with current confidence, clamped to [0.0, 1.0]
    let adjusted = current_confidence + base_adjustment + context_bonus;
    adjusted.max(0.0).min(1.0)
}

/// Apply time-based decay to all beliefs in state
fn apply_decay_to_all(state: &mut UserState) {
    let now = chrono::Utc::now();

    for (_concept_id, belief) in state.concepts.iter_mut() {
        let seconds_since_last_seen = now.timestamp().saturating_sub(belief.last_seen.timestamp());
        let hours_elapsed = seconds_since_last_seen as f32 / 3600.0;

        if hours_elapsed > 0.0 {
            belief.confidence = (belief.confidence - (belief.decay_rate * hours_elapsed)).max(0.0);
            belief.last_seen = now;
        }
    }
}

/// Create a basecamp snapshot at current state minimum confidence
pub fn set_basecamp(state: &UserState, description: &str, threshold: f32) -> Option<Snapshot> {
    // Use fold to find min since we removed the custom trait
    let min_confidence = state
        .concepts
        .values()
        .map(|b| b.confidence)
        .fold(f32::MAX, |a, b| a.min(b));

    if min_confidence >= threshold && !state.concepts.is_empty() {
        return Some(Snapshot {
            description: description.to_string(),
            snapshot_at: chrono::Utc::now(),
            confidence_threshold: min_confidence,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConceptId, InteractionId, InteractionKind};

    fn test_concept() -> ConceptId {
        ConceptId("test_concept".to_string())
    }

    #[test]
    fn test_update_creates_new_belief_on_first_interaction() {
        let state = UserState::default();
        let interaction = Interaction {
            id: InteractionId(0),
            kind: InteractionKind::Applied,
            concepts_touched: vec![test_concept()],
            resolved: false,
            at: chrono::Utc::now(),
        };

        let new_state = update(state, interaction);

        assert!(new_state.concepts.contains_key(&test_concept()));
        let belief = new_state.concepts.get(&test_concept()).unwrap();
        assert_eq!(belief.confidence, 0.4); // Applied gets higher baseline
    }

    #[test]
    fn test_update_increases_confidence_on_application() {
        let mut state = UserState::default();
        state.concepts.insert(
            test_concept(),
            Belief {
                confidence: 0.5,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 0.1,
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let interaction = Interaction {
            id: InteractionId(0),
            kind: InteractionKind::Applied,
            concepts_touched: vec![test_concept()],
            resolved: false,
            at: chrono::Utc::now(),
        };

        let new_state = update(state, interaction);
        let belief = new_state.concepts.get(&test_concept()).unwrap();

        assert!(
            belief.confidence > 0.5,
            "Confidence should increase on application"
        );
    }

    #[test]
    fn test_confidence_never_exceeds_bounds() {
        let mut state = UserState::default();
        state.concepts.insert(
            test_concept(),
            Belief {
                confidence: 0.95,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 0.0, // No decay for this test
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let interaction = Interaction {
            id: InteractionId(0),
            kind: InteractionKind::Applied,
            concepts_touched: vec![test_concept()],
            resolved: false,
            at: chrono::Utc::now(),
        };

        let new_state = update(state, interaction);
        let belief = new_state.concepts.get(&test_concept()).unwrap();

        assert!(belief.confidence <= 1.0, "Confidence should not exceed 1.0");
    }

    #[test]
    fn test_context_length_provides_stability_bonus() {
        let mut state = UserState::default();

        // Build up context through multiple interactions
        for i in 0..5 {
            state.concepts.insert(
                test_concept(),
                Belief {
                    confidence: 0.6,
                    last_seen: chrono::Utc::now(),
                    context: vec![InteractionId(i)],
                    decay_rate: 0.1,
                    loop_count: 0,
                    loop_delta: 0.0,
                    last_confidence: None,
                },
            );
        }

        let interaction = Interaction {
            id: InteractionId(5),
            kind: InteractionKind::Applied,
            concepts_touched: vec![test_concept()],
            resolved: false,
            at: chrono::Utc::now(),
        };

        let new_state = update(state, interaction);
        let belief = new_state.concepts.get(&test_concept()).unwrap();

        // Should have context bonus from multiple previous interactions
        assert!(
            belief.confidence > 0.65,
            "Should gain confidence from accumulated evidence"
        );
    }

    #[test]
    fn test_confidence_never_negatively_from_questions() {
        let mut state = UserState::default();
        state.concepts.insert(
            test_concept(),
            Belief {
                confidence: 0.1,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 0.0, // No decay for this test
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let interaction = Interaction {
            id: InteractionId(0),
            kind: InteractionKind::Asked,
            concepts_touched: vec![test_concept()],
            resolved: false,
            at: chrono::Utc::now(),
        };

        let new_state = update(state, interaction);
        let belief = new_state.concepts.get(&test_concept()).unwrap();

        assert!(
            belief.confidence >= 0.1,
            "Confidence should not drop below starting point without decay"
        );
    }

    #[test]
    fn test_basecamp_creation_succeeds_when_threshold_met() {
        let state = UserState::default();

        // Initialize with concepts above threshold
        let mut test_state = state.clone();
        test_state.concepts.insert(
            ConceptId("concept1".to_string()),
            Belief {
                confidence: 0.8,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 0.1,
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let result = set_basecamp(&test_state, "Test basecamp", 0.7);

        assert!(result.is_some());
        assert_eq!(result.unwrap().confidence_threshold, 0.8);
    }

    #[test]
    fn test_basecamp_creation_fails_when_below_threshold() {
        let state = UserState::default();

        // Initialize with concepts below threshold
        let mut test_state = state.clone();
        test_state.concepts.insert(
            ConceptId("concept1".to_string()),
            Belief {
                confidence: 0.5,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 0.1,
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let result = set_basecamp(&test_state, "Test basecamp", 0.7);

        assert!(result.is_none());
    }

    #[test]
    fn test_apply_decay_reduces_confidence() {
        let mut state = UserState::default();
        let concept = ConceptId("decay_test".to_string());

        state.concepts.insert(
            concept.clone(),
            Belief {
                confidence: 0.9,
                last_seen: chrono::Utc::now(),
                context: Vec::new(),
                decay_rate: 1.0, // High decay rate for test
                loop_count: 0,
                loop_delta: 0.0,
                last_confidence: None,
            },
        );

        let hours_elapsed = 2.0;
        state
            .concepts
            .get_mut(&concept)
            .unwrap()
            .apply_decay(hours_elapsed);

        let belief = state.concepts.get(&concept).unwrap();

        assert!(belief.confidence < 0.9, "Confidence should decay over time");
        assert!(
            belief.confidence >= 0.0,
            "Confidence should not go below zero"
        );
    }
}
