use nu_drift::{AgentOrchestrator, State};
use nu_drift::automation::AutoTracker;
use nu_drift::types::{Interaction, InteractionKind};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {\n    // Initialize state\n    let state = Arc::new(Mutex::new(State::new()));\n    let tracker = AutoTracker::new(state.clone());\n    \n    // Create orchestrator\n    let orchestrator = AgentOrchestrator::new(state.clone());\n    \n    // Simulate user interaction\n    let user_message = "I need to build a Rust program with Docker";\n    println!("User: {}", user_message);\n    \n    // Run interaction (automatic tracking)\n    let response = orchestrator.run_interaction(user_message).await;\n    println!("Agent: {}", response);\n    \n    // Check state\n    let state = orchestrator.get_state().await;\n    println!("\n--- State Summary ---");\n    \n    // Show tracked concepts\n    println!("Concepts: {}", state.concepts.len());\n    \n    // Show stuck concepts\n    let stuck = orchestrator.get_stuck_concepts().await;\n    if stuck.is_empty() {\n        println!("No stuck concepts");\n    } else {\n        println!("Stuck concepts: {:?}", stuck);\n    }\n    \n    // Show last applied\n    let last = orchestrator.last_applied(3).await;\n    println!("Last applied interactions: {}", last.len());\n}
