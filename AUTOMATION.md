Automatic Concept Tracking System
=========================

Overview
--------
The automation system automatically tracks user interactions and concepts without requiring manual intervention.

Components
----------
1. Concept Extractor (src/automation/concept_extractor.rs)
   - Automatically detects concepts from user messages
   - Uses simple pattern matching (no complex regex)
   - Extracts concepts like: programming languages, frameworks, patterns

2. Auto Tracker (src/automation/auto_tracker.rs)
   - Manages the state and automatically records interactions
   - Creates initial beliefs for new concepts
   - Tracks concept existence to avoid duplicates

3. Agent Orchestrator (src/automation/orchestrator.rs)
   - Coordinates the automatic tracking system
   - Adds context about stuck/low-confidence concepts
   - Bridges between user interactions and state management

Usage Example
-------------
```rust
use nu_drift::{AgentOrchestrator, State};
use nu_drift::automation::AutoTracker;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // Initialize state
    let state = Arc::new(Mutex::new(State::new()));
    let tracker = AutoTracker::new(state.clone());
    
    // Create orchestrator
    let orchestrator = AgentOrchestrator::new(state.clone());
    
    // User interaction (automatic tracking)
    let response = orchestrator.run_interaction("I need to build a Rust program").await;
    
    // Query state
    let stuck = orchestrator.get_stuck_concepts().await;
    println!("Stuck concepts: {:?}", stuck);
}
```

Key Features
------------
- Automatic Detection: Concepts are detected from text patterns
- Zero Manual Tracking: No need to manually specify concepts
- Context Awareness: Stuck/low-confidence concepts are automatically added to context
- State Persistence: State can be saved and loaded from JSON

Concept Extraction Patterns
---------------------------
The system detects these concept types:
- Programming languages: rust, python, javascript, typescript
- Frameworks: docker, kubernetes
- Architecture patterns: api, microservice, event-driven
- Development concepts: cli, script, test, debug

Integration with Agent
----------------------
To integrate with an AI agent:
1. Create an AgentOrchestrator instance
2. Call run_interaction() for each user message
3. The system automatically tracks concepts and adds context
4. Agent receives context-aware responses
5. State is automatically updated

Running the Demo
----------------
cargo run --example auto_demo
