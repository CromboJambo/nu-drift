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
        print "No state found at " + $state_file + ". Run the agent first."
        return
    }

    # Parse JSON and extract trajectory
    let state = (open $state_file | from json)

    # Filter for Applied interactions, take most recent N
    let applied = ($state.trajectory
        | where kind == "Applied"
        | sort-by at --reverse
        | first $count)

    if ($applied | is-empty) {
        if $format == "concise" {
            print "No interactions recorded yet."
        } else {
            print "No applied interactions found (showing last " + $count + ")."
        }
        return
    }

    if $format == "json" {
        $applied | to json --indent 2
    } else if $format == "concise" {
        $applied | each {|i|
            $i.at + ": " + ($i.concepts_touched | str join ", ")
        } | str join "\n"
    } else {
        # Table format with interaction details
        $applied | each {|i|
            let concept_count = ($i.concepts_touched | length)
            {
                id: $i.id,
                timestamp: $i.at,
                concepts: ($i.concepts_touched | str join ', '),
                concept_count: $concept_count,
                resolved: (if $i.resolved { "✓" } else { "" })
            }
        } | table --expand
    }

    # Optional context: what happened before these applications?
    if ($state.trajectory | where kind == "Asked" or kind == "Confused" | length) > 0 {
        print "\n--- Context ---"
        let questions = ($state.trajectory
            | where (kind == "Asked") or (kind == "Confused")
            | sort-by at --reverse
            | first 3)

        if $format == "concise" {
            print $"Previous questions/confusion: ($questions | length)"
        } else {
            print $"Recent questions and confusion ({($questions | length)}):"
            $questions | each {|q|
                print $"  - ($q.at): ($q.concepts_touched | str join ', ')"
            }
        }
    }

    # Basecamp reference if exists
    if $state.basecamp != null {
        print "\n--- Current Basecamp ---"
        if $format == "concise" {
            $"Located at: ($state.basecamp.description) (confidence ≥ $($state.basecamp.confidence_threshold | into string --precision 2))"
        } else {
            print $"Description: ($state.basecamp.description)"
            print $"Established: ($state.basecamp.snapshot_at)"
            print $"Minimum confidence required: ($state.basecamp.confidence_threshold | into string --precision 2)"
        }
    }
}
