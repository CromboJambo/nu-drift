//! Nu Drift - Async Learning State Manager
//!
//! This is the demo binary for nu-drift. It runs a small interaction loop
//! with explicit tagging to show how state evolves.

use nu_drift::State;
use nu_drift::types::{ConceptId, InteractionKind};

/// Run multiple interactions and check for stuck concepts (async demo)
async fn run_demo(state: &State) {
    println!("=== Async Learning Demo ===\n");

    let concept1 = ConceptId("async_await".to_string());

    println!("Recording initial interactions...");

    // First, explicitly tag the concept to create it in the state
    state
        .record_interaction(InteractionKind::Applied, &[concept1.clone()])
        .await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Then simulate repeated asking (no progress, loop counter increases)
    for _ in 0..8 {
        state
            .record_interaction(InteractionKind::Asked, &[concept1.clone()])
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
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
