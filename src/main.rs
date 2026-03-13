//! Nu Drift - Learning Agent Framework
//!
//! The core philosophy: invert the reward structure. The tool serves the user
//! as their self-authored basecamp. It never performs certainty it doesn't have.
//! Neither does the user.

mod types;
mod update;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use types::{ConceptId, InteractionKind, UserState};

/// Configuration for the agent loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Model identifier (OpenRouter-compatible APIs)
    pub model: String,
    /// API endpoint URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_endpoint: Option<String>,
    /// Minimum confidence threshold for basecamp
    pub basecamp_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "anthropic/claude-3-haiku".to_string(),
            api_endpoint: None,
            basecamp_threshold: 0.7,
        }
    }
}

/// State persistence manager
struct StateManager {
    state_file: String,
    state: UserState,
}

impl StateManager {
    fn new(state_file: &str) -> Self {
        let state = match fs::read_to_string(state_file) {
            Ok(json) => UserState::from_json(&json).unwrap_or_default(),
            Err(_) => UserState::default(),
        };

        Self {
            state_file: state_file.to_string(),
            state,
        }
    }

    fn load(&mut self) -> &UserState {
        if let Ok(json) = fs::read_to_string(&self.state_file) {
            if let Ok(parsed) = UserState::from_json(&json) {
                self.state = parsed;
            }
        }
        &self.state
    }

    fn save(&self) -> io::Result<()> {
        let json = self.state.to_json().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Serialization error: {}", e))
        })?;
        fs::write(&self.state_file, json)
    }

    fn record_interaction(
        &mut self,
        kind: InteractionKind,
        concepts_touched: &[ConceptId],
    ) -> types::InteractionId {
        let _id = types::InteractionId(self.state.trajectory.len() as u64);
        self.state.record_interaction(kind, concepts_touched)
    }

    fn get_state_mut(&mut self) -> &mut UserState {
        &mut self.state
    }
}

/// Synchronous agent loop skeleton
///
/// This is the core reasoning loop. It:
/// 1. Loads current state
/// 2. Queries for what needs attention (via tools)
/// 3. Makes API call to get reasoning/response
/// 4. Records result as interaction
/// 5. Updates state deterministically
fn agent_loop(
    config: &Config,
    manager: &mut StateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nu Drift Agent Loop ===");
    println!("Model: {}", config.model);

    // Load current state
    let _state = manager.load();

    // Query layer: what needs revisiting?
    if let Some(concepts) = query_needs_revisiting(manager.get_state_mut(), 0.5) {
        println!("\nConcepts needing revisiting (confidence < 0.5):");
        for (concept_id, belief) in concepts {
            println!(
                "  - {}: {:.2} (last seen: {})",
                concept_id.0, belief.confidence, belief.last_seen
            );
        }
    } else {
        println!("\nNo concepts currently below confidence threshold.");
    }

    // Query layer: what did they actually build?
    if let Some(latest) = query_last_built(manager.get_state_mut(), 3) {
        println!("\nLast {} applied interactions:", latest.len());
        for interaction in latest {
            println!(
                "  - {:?} at {}, concepts: {:?}",
                interaction.kind, interaction.at, interaction.concepts_touched
            );
        }
    }

    // In a real implementation, this would be where we call the API
    // For now, we simulate with interactive input
    println!("\n--- Interactive Demo ---");
    print!("Enter concept name (or 'q' to quit): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input == "q" || input.is_empty() {
        println!("Exiting demo. State saved.");
        manager.save()?;
        return Ok(());
    }

    // Record interaction based on user intent
    print!(
        "What happened with '{}'?\n  [1] Asked a question\n  [2] Expressed confusion\n  [3] Applied knowledge\nChoice: ",
        input
    );
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let kind = match choice.trim() {
        "1" => InteractionKind::Asked,
        "2" => InteractionKind::Confused,
        "3" => InteractionKind::Applied,
        _ => {
            println!("Invalid choice. Exiting.");
            manager.save()?;
            return Ok(());
        }
    };

    let concept_id = ConceptId(input.to_string());
    manager.record_interaction(kind, &[concept_id]);

    // Apply decay to simulate time passing (optional in demo)
    if kind == InteractionKind::Applied {
        println!("Knowledge applied. Confidence updated.");
    } else {
        println!("Interaction recorded. State persists.");
    }

    manager.save()?;

    Ok(())
}

/// Query: what needs revisiting?
fn query_needs_revisiting(
    state: &UserState,
    threshold: f32,
) -> Option<Vec<(&ConceptId, &types::Belief)>> {
    let concepts = state.needs_revisiting(threshold);
    if concepts.is_empty() {
        None
    } else {
        Some(concepts)
    }
}

/// Query: what did they actually build?
fn query_last_built(state: &UserState, n: usize) -> Option<Vec<types::Interaction>> {
    let applied = state.last_applied(n);
    if applied.is_empty() {
        None
    } else {
        Some(applied.into_iter().cloned().collect())
    }
}

/// Load configuration from file or use defaults
fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    if let Ok(json) = fs::read_to_string(path) {
        serde_json::from_str(&json).map_err(|e| format!("Config parse error: {}", e).into())
    } else {
        println!("No config file at {}, using defaults", path);
        Ok(Config::default())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state_file = "state.json";
    let config_path = "config.toml";

    println!("Nu Drift v0.1.0");
    println!("Learning agent with calibrated uncertainty\n");

    // Try to load config, fall back to defaults
    let mut config = match load_config(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: {}", e);
            Config::default()
        }
    };

    // Allow environment variable override for API endpoint (for testing)
    if let Ok(endpoint) = std::env::var("NU_DRIFT_API_ENDPOINT") {
        config.api_endpoint = Some(endpoint);
    }

    let mut manager = StateManager::new(state_file);

    // Run agent loop
    match agent_loop(&config, &mut manager) {
        Ok(()) => println!("\nSession complete."),
        Err(e) => eprintln!("Error in agent loop: {}", e),
    }

    Ok(())
}
