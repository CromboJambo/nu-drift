#!/usr/bin/env nu
# Nu Drift - Query: Is basecamp current?
#
# Checks whether the user's self-authored stable point is still valid
# and reflects their current understanding level
#
# The basecamp is not "completed module 3" — it's a snapshot the user
# authors themselves: "I understood this well enough to leave from here."

export def main [
    --threshold=-t: float = null  # Override confidence threshold (default: use stored)
    --format=-f: string = "table"  # Output format: table | json | concise
] {
    # Load state from JSON file
    let state_file = "../state.json"

    if not ($state_file | path exists) {
        print $"No state found at ($state_file). Run the agent first."
        return
    }

    # Parse JSON and extract state
    let state = (open $state_file | from json)

    match $state.basecamp {
        null => {
            match $format {
                "concise" => print "No basecamp established yet."
                _ => print [
                    "No stable point set."
                    ""
                    "You can establish a basecamp when you feel ready to mark:"
                    "  'I understood this well enough to leave from here.'"
                ] | str join "\n"
            }
            return
        }
        $basecamp => {
            # Basecamp exists — check if it's still valid

            # Get current minimum confidence across all concepts
            let min_confidence = ($state.concepts.values | get confidence | min)

            match $format {
                "json" => {
                    {
                        basecamp: $basecamp,
                        current_minimum_confidence: $min_confidence,
                        status: (if $min_confidence >= $basecamp.confidence_threshold {
                            "stable"
                        } else {
                            "drifted"
                        }),
                        drift_amount: (($basecamp.confidence_threshold - $min_confidence) | into string --precision 2)
                    } | to json --pretty
                }
                "concise" => {
                    let status = if $min_confidence >= $basecamp.confidence_threshold {
                        "stable"
                    } else {
                        "drifted"
                    }

                    print $"Basecamp: ($basecamp.description)"
                    print $"Status: ($status) (current min: ($min_confidence | into string --precision 2))"

                    if $status == "drifted" {
                        print "Consider revisiting before re-establishing."
                    } else {
                        print "Current understanding meets basecamp threshold."
                    }
                }
                _ => {
                    # Table format with full details
                    let status = if $min_confidence >= $basecamp.confidence_threshold {
                        "stable"
                    } else {
                        "drifted"
                    }

                    print [
                        "--- Basecamp Status ---"
                        ""
                        $"Description: ($basecamp.description)"
                        $"Established: ($basecamp.snapshot_at)"
                        $"Minimum confidence at basecamp: ($basecamp.confidence_threshold | into string --precision 2)"
                        ""
                        $"Current minimum confidence across all concepts: ($min_confidence | into string --precision 2)"
                        $"Status: ($status)"
                    ] | str join "\n"

                    if $status == "drifted" {
                        print [
                            ""
                            "--- Recommendation ---"
                            $"You've drifted below your basecamp threshold by ((($basecamp.confidence_threshold - $min_confidence) * 100) | into string --precision 0)%"

                            "Options:"
                            "  1. Revisit concepts that need work (see: needs_revisiting.nu)"
                            "  2. Establish a new basecamp at current level"
                        ] | str join "\n"
                    } else {
                        print [
                            ""
                            "--- Recommendation ---"
                            "Your understanding is stable relative to your basecamp."

                            "If you've learned beyond this point, consider establishing a new basecamp to mark progress."
                        ] | str join "\n"
                    }
                }
            }
        }
    }
}
