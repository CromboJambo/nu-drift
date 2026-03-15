use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::types::{ConceptId, InteractionKind};
use crate::State;

pub struct AutoTracker {
    state: Arc<Mutex<State>>,
    known_concepts: Arc<RwLock<Vec<ConceptId>>>,
}

impl AutoTracker {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self {
            state,
            known_concepts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn track_interaction(&self, interaction: crate::types::Interaction) {
        let concepts = extract_concepts(&interaction.to_string(), interaction.kind);

        for concept in concepts {
            if !self.concept_exists(&concept).await {
                self.record_initial_concept(&concept).await;
            }
        }

        let mut state = self.state.lock();
        state.record_interaction(interaction.kind, &concepts);
    }

    async fn record_initial_concept(&self, concept: &ConceptId) {
        let mut state = self.state.lock();
        let belief = crate::types::Belief::new(0.5, 0.1);
        state.concepts.insert(concept.clone(), belief);

        let mut known = self.known_concepts.write().await;
        if !known.contains(concept) {
            known.push(concept.clone());
        }
    }

    async fn concept_exists(&self, concept: &ConceptId) -> bool {
        let known = self.known_concepts.read().await;
        known.iter().any(|c| c == concept)
    }

    pub async fn get_state(&self) -> crate::types::UserState {
        self.state.lock().await.get_state().await
    }

    pub async fn get_stuck_concepts(&self) -> Vec<(ConceptId, u32)> {
        self.get_state().await.get_stuck_concepts()
    }

    pub async fn needs_revisiting(&self, threshold: f32) -> Vec<(ConceptId, crate::types::Belief)> {
        self.get_state().await.needs_revisiting(threshold)
    }

    pub async fn last_applied(&self, n: usize) -> Vec<crate::types::Interaction> {
        self.get_state().await.last_applied(n)
    }

    pub async fn check_basecamp(&self) -> Option<crate::types::Snapshot> {
        self.get_state().await.basecamp.clone()
    }
}
