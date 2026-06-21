use nu_drift::types::{ConceptId, InteractionKind};
use nu_drift::State;

#[tokio::main]
async fn main() {
    let state = State::new();

    let concepts = vec![
        ConceptId("rust_programming".to_string()),
        ConceptId("docker".to_string()),
    ];

    state
        .record_interaction(InteractionKind::Asked, &concepts)
        .await;

    let json = state.to_json().await.unwrap();
    println!("{}", json);
}
