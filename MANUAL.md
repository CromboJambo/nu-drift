Manual-First Flow (Explicit Tagging)
===================================

Overview
--------
This project uses explicit concept tags. The system does not guess concepts
from text. If a concept matters, it is tagged deliberately.

Why this approach
-----------------
- Lower complexity: no extraction heuristics, no false positives.
- Higher signal: every concept is intentional.
- Easier debugging: you can trace every concept to an explicit tag.

What the user does
------------------
When a learning event happens, the user (or agent UI) supplies:
- `InteractionKind` (Asked | Confused | Applied | Stuck)
- `concepts_touched` (explicit ConceptId list)

Example workflow
----------------
1. User asks: "I need to build a Rust program with Docker."
2. You decide which concepts matter:
   - `rust_programming`
   - `docker`
3. Record the interaction explicitly:
   - kind: `Asked`
   - concepts: `["rust_programming", "docker"]`

The system updates state deterministically via `update()` and records the
interaction in the trajectory.

Minimal tagging conventions
---------------------------
- Use stable, lowercase IDs with underscores, e.g. `rust_programming`.
- Keep tags sparse: only what matters for the learning trajectory.
- If unsure, tag fewer concepts and refine later.

Demo
----
Run the explicit-tag demo:
`cargo run`

This uses `State::record_interaction()` with explicit `ConceptId` values.
