## How to work on this step

This is one step of the plan in `plan/README.md`: a linear-logic proof-search
suite the author uses for research and teaching, built step by step by
separate sessions. Later steps build on what you leave behind, so the report
you write and the rules under `.claude/rules/` matter as much as the code.

The numbered items above are requirements, not an order of work: choose the
order and the design, and read the spec's sections named above before
deciding rather than working from memory. When you have enough information
to act, act; do not re-litigate decisions the plan records, and where you
weigh a choice, pick one and say why in the report rather than surveying the
alternatives.

Don't add features, refactor, or introduce abstractions beyond what the step
requires. Don't design for hypothetical future requirements beyond what the
plan names. Avoid premature abstraction, and avoid half-finished
implementations too. Don't add error handling or validation for scenarios
that cannot happen; trust internal invariants and validate at the system
boundaries (parsing, deserialization, CLI input). If, while working or
testing, you find a pre-existing bug, a performance concern, or behaviour the
step doesn't mention, don't fix, optimize or extend it in this change unless
the requested behaviour cannot work without it; report it as a follow-up.
Where the step is ambiguous, implement the reading its wording, the spec and
the surrounding code most directly support, state that assumption in the
report, and don't build for the other readings as well.

Test as necessary, not as much as possible. Commit tests only where the step
names a behaviour to pin or the repository already keeps tests for this kind
of change, sized like the neighbouring test files, roughly one focused test
per stated behaviour; scratch checks and exploratory scripts stay out of the
repository. Verify your work however you like as you go (run the check
commands while you build, not only at the end), and before reporting
progress audit each claim against a tool result from this session: only
report work you can point to evidence for, say explicitly what is not yet
verified, and if a check fails say so with its output. Prefer targeted edits
to whole-file rewrites where the result is the same.

Delegate independent work to sub-agents and keep working while they run:
the `crate-source-explorer` agent for any question about a dependency's API
at the pinned version, and a fresh-context reviewer for a soundness-critical
piece (a checker, a criterion, a prune) before you call it done. Intervene
if a sub-agent goes off track.

Record what a future session must know and cannot see in the code in
`.claude/rules/core.md` (invariants, why a choice was made, what a check
cannot catch), one point per bullet, and in your step report. Update an
existing note rather than adding a duplicate; delete what turns out to be
wrong. Code comments and doc comments never mention this session, the
prompt, the plan, its steps or its decision numbers: they say what the
code does or why, in terms a reader of the repository alone understands.
The plan and the reports are where the organisation of the work lives.

`README.md` is the public face of the repository and must describe what
exists after your step: extend its usage section with the commands, flags
and formats you added (real invocations with their output), and keep its
"What exists and what is planned" section true, moving what you built from
planned to built. Do not mention the plan, its steps or these sessions
there.

In the report and in your final message, lead with the outcome, then the
decisions, then what is left open. Complete sentences, no working
shorthand, and every file, type or flag you name gets its own clause saying
what it is.
