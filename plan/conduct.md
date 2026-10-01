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

When you design a library-level feature whose output people see, design
its configuration for every front end at once (plan decision D15): one
plain-data options value with defaults and serde, which the CLI maps its
flags onto, the web front end holds as JSON and any other wrapper reuses;
no choice a user might want to vary hidden in a constant. Say in the
report which options exist and how each front end would set them.

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

The machine is shared with its owner's other work. Every scratch program,
yours or a sub-agent's, runs in a memory-capped scope of its own
(`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 …`) with
its enumerations bounded by size, because an unbounded checker once took
62 GB and the kernel killed the whole terminal with the session in it;
say so in every sub-agent's brief. Anything that runs for more than a few
minutes runs detached from the terminal (a systemd user unit) so that it
survives the session. Do not use every core or run `bench/baseline.sh`
unless the step says the machine is free. A benchmark or probe run by
day that the step does not name is asked for first, however small and
on whichever cores, with its purpose, its duration and the cores it
takes, and waits for the author's yes; one step ran three hours of
probes on the efficiency cores to choose a list and had to be stopped.

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

Everything you produce in one reply, including any reasoning or drafting
before the reply, counts toward a single limit of about 128 000 tokens. If
that limit is reached before the reply is finished, the whole turn is lost
and the work has to start over. Composing an entire module in full as
reasoning and then again as a reply would double the length of the turn
without improving the result, so do not do that: spend the reasoning on
understanding the requirements, checking the code your design depends on,
and settling the structure and the hard decisions, then write the code
into files and let the compiler and the tests take it from there.

In the report and in your final message, lead with the outcome, then the
decisions, then what is left open. Complete sentences, no working
shorthand, and every file, type or flag you name gets its own clause saying
what it is.
