#!/usr/bin/env nu
# Nu Drift - Query: What needs revisiting?
#
# Lists all concepts where confidence has decayed below threshold
# This is the primary diagnostic for when to focus learning effort

export def main [
    --threshold=-t: int = 0.5  # Confidence threshold (lower = more forgiving)
    --format=-f: string = "table"  # Output format: table | json | concise
] {
    # Load state from JSON file
    let state_file = "../state.json"

    if not ($state_file | path exists) {
        print $"No state found at ($state_file). Run the agent first."
        return
    }

    # Parse JSON and extract concepts
    let state = (open $state_file | from json)

    # Filter by confidence threshold
    let needs_review = ($state.concepts | items | where confidence < $threshold)

    if ($needs_review | is-empty) {
        if $format == "concise" {
            print $"✓ All concepts above confidence threshold (≥$threshold)"
        } else {
            print $"No concepts currently below confidence threshold ($threshold)."
        }
        return
    }

    match $format {
        "json" => {
            $needs_review | select concept_id confidence last_seen decay_rate context | to json --pretty
        }
        "concise" => {
            $needs_review | each {|c| $"($c.concept_id): ($c.confidence | into string --precision 2)" } | str join "\n"
        }
        _ => {
            # Table format with human-readable timestamps
            $needs_review | each {|c|
                let hours_ago = (
                    (date now) - ($c.last_seen | into datetime) | duration to-text | str replace -r '[0-9]+(years?|months?|days?)' '' | str trim
                )

                {
                    concept: $c.concept_id,
                    confidence: ($c.confidence | into string --precision 2),
                    last_seen: $"($hours_ago) ago",
                    decay_rate: ($c.decay_rate | into string --precision 2),
                    context_count: ($c.context | length)
                }
            } | sort-by confidence | table --expand
        }
    }

    # Optional: show basecamp status if exists
    if $state.basecamp != null {
        print "\n--- Basecamp Status ---"
        print $"Current basecamp: ($state.basecamp.description)"
        print $"Set at: ($state.basecamp.snapshot_at | date to-table | get --raw table | str join ' ')"
        print $"Minimum confidence at basecamp: ($state.basecamp.confidence_threshold | into string --precision 2)"

        if ($needs_review | length) > 0 {
            print "Note: Some concepts have drifted below basecamp threshold."
        }
    }
}
```

<tool_call>
<function=create_file>
<parameter=path>
nu-drift/scripts/last_built.nu
</parameter>
<parameter=mode>
create
</parameter>
</function>
</tool_call>
