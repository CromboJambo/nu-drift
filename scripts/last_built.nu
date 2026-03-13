#!/usr/bin/env nu
# Nu Drift - Query: What did they actually build?
#
# Shows the most recent interaction records where knowledge was applied
# This is about practice, not theory — what got built in the real world

export def main [
    --count=-c: int = 5  # Number of interactions to show (default: 5)
    --format=-f: string = "table"  # Output format: table | json | concise
] {
    # Load state from JSON file
    let state_file = "../state.json"

    if not ($state_file | path exists) {
        print $"No state found at ($state_file). Run the agent first."
        return
    }

    # Parse JSON and extract trajectory
    let state = (open $state_file | from json)

    # Filter for Applied interactions, take most recent N
    let applied = ($state.trajectory
        | where kind == "Applied"
        | sort-by at --descending
        | first $count)

    if ($applied | is-empty) {
        match $format {
            "concise" => print "No interactions recorded yet."
            _ => print $"No applied interactions found (showing last $count)."
        }
        return
    }

    match $format {
        "json" => {
            $applied | to json --pretty
        }
        "concise" => {
            $applied | each {|i|
                $"($i.at): ($i.concepts_touched | str join ', ')"
            } | str join "\n"
        }
        _ => {
            # Table format with interaction details
            $applied | each {|i|
                let concept_count = ($i.concepts_touched | length)
                {
                    id: $i.id,
                    timestamp: $i.at,
                    concepts: ($i.concepts_touched | str join ", "),
                    concept_count: $concept_count,
                    resolved: (if $i.resolved then "✓" else "")
                }
            } | table --expand
        }
    }

    # Optional context: what happened before these applications?
    if ($state.trajectory | where kind == "Asked" or kind == "Confused" | length) > 0 {
        print "\n--- Context ---"
        let questions = ($state.trajectory
            | where (kind == "Asked") or (kind == "Confused")
            | sort-by at --descending
            | first 3)

        match $format {
            "concise" => {
                print $"Previous questions/confusion: ($questions | length)"
            }
            _ => {
                print $"Recent questions and confusion ({($questions | length)}):"
                $questions | each {|q|
                    print $"  - ($q.at): ($q.concepts_touched | str join ', ')"
                }
            }
        }
    }

    # Basecamp reference if exists
    if $state.basecamp != null {
        print "\n--- Current Basecamp ---"
        match $format {
            "concise" => {
                $"Located at: ($state.basecamp.description) (confidence ≥ $($state.basecamp.confidence_threshold | into string --precision 2))"
            }
            _ => {
                print $"Description: ($state.basecamp.description)"
                print $"Established: ($state.basecamp.snapshot_at)"
                print $"Minimum confidence required: ($state.basecamp.confidence_threshold | into string --precision 2)"
            }
        }
    }
}
