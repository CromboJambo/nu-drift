# Project Starter Prompt

## What you're building

A learning agent framework where the agent's model of the user is structured,
typed, honest data — not a vibe accumulated in a context window. The core
philosophy: invert the reward structure. The tool serves the user as their
self-authored basecamp. It never performs certainty it doesn't have. Neither
does the user.

---

## Stack

- **Rust** — agent core, API calls, data types
- **Nushell** — query layer, tool scripts, structured pipeline glue
- **Anthropic API** — the reasoning loop

These share a design philosophy (Jonathan Turner, same author of both Nu and
core Rust tooling): make the data model explicit, surface divergence honestly,
don't paper over weird behavior. The stack is not incidental — it enforces the
values of the project.

---

## Core data model

```rust
struct UserState {
    concepts:   HashMap<ConceptId, Belief>,
    trajectory: Vec<Interaction>,
    basecamp:   Option<Snapshot>,
}

struct Belief {
    confidence:  f32,        // 0.0–1.0, not bool
    last_seen:   Instant,
    context:     Vec<InteractionId>,  // proof by implication
    decay_rate:  f32,        // knowledge untouched loses confidence
}

struct Interaction {
    kind:             InteractionKind,  // Asked | Confused | Applied
    concepts_touched: Vec<ConceptId>,
    resolved:         bool,
    at:               Instant,
}

// pure function — old state in, new state out
fn update(state: UserState, interaction: Interaction) -> UserState
```

`confidence: f32` not `knows: bool` is the whole philosophy in one field.
`basecamp` is not "completed module 3" — it's a snapshot the user authors
themselves: *"I understood this well enough to leave from here."*

---

## Agent loop (s01 pattern, Rust)

```rust
loop {
    let response = client.messages(model, system, messages, tools).await?;
    messages.push(response.content);

    if response.stop_reason != StopReason::ToolUse {
        break;
    }

    let results = dispatch_tools(&response.content);
    messages.push(results);
}
```

Start with s01–s03 from `github.com/shareAI-lab/learn-claude-code` as the
conceptual reference. Port the loop to Rust with `reqwest` + `serde_json`.
Nu scripts handle tool result queries over `UserState`.

---

## Confidence contract

Both the agent and the user operate under the same rule:

> Calibrated uncertainty is a first-class output. The agent says "I think
> you're ~70% solid on this" not "learned: true." The user says "I'm not sure
> about this yet" and the system treats that as signal, not failure.

The agent must not perform certainty to make the user feel safe. The users who
need false certainty are not the users this is for.

---

## What "learning" means here

Not completing loops a fixed number of times. Not passing a check. Learning
is when the same pattern appears in enough different contexts that it stops
feeling like a pattern. The tool's job is not to optimize that process — it's
to not obstruct it. Hold the context. Don't make the user re-explain
themselves. Don't reward performance of understanding over actual
understanding.

Ignorance of ignorance does not equal low intelligence. The friction of asking
a question you didn't know you needed to ask is where real understanding gets
built. The agent rewards that friction, not the absence of it.

---

## Nu as query layer

```nushell
# what needs revisiting?
$state.concepts | where confidence < 0.5 | sort-by last_seen

# what did they actually build?
$state.trajectory | where kind == Applied | last 5

# is basecamp current?
$state.basecamp | if $in == null { "no stable point yet" } else { $in.snapshot_at }
```

Structured data all the way down. No artisanal text parsing. The same
philosophy as the type system — data has shape, queries respect that shape.

---

## What this is not

- Not a gamified learning platform
- Not a tutorial with checkboxes
- Not a tool that assumes you know what you don't know
- Not a tool that papers over POSIX-style weirdness to feel familiar

---

## Starting point

1. Implement the agent loop in Rust (sync first, async later)
2. Define `UserState`, `Belief`, `Interaction` as serde-serializable types
3. Write `fn update()` as a pure function
4. Write Nu scripts to query state
5. Wire a single tool: `record_interaction`
6. Build from there — one mechanism at a time, one motto per session

The mess is the point. Every interesting idea is a collision of influences
the builder can't fully untangle. Build it anyway.
