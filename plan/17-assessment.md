# Step 17: where the project stands, and the plan from here

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), no pushing. This step writes no code. It has two parts
with the author between them: an assessment that ends in the decisions
only the author can make, and, once the author has made them, the plan of
the steps from 18 on. Read before you start, and read the code itself, not
only what is written about it:

- `plan/README.md` from its first line to its last: the step table, every
  design decision (D1 to D15) and the whole Status log, which says what
  each step's review accepted, corrected and left open.
- Every report in `plan/reports/`, above all the three about numbers:
  `14-benchmarks.md` (the first baseline), `15-performance.md` (what the
  performance pass changed) and `16-baseline.md` (the second baseline and
  the comparison).
- `plan/later.md`: the candidates for further work and the follow-up
  lists, which are this step's subject.
- `proof-search-specifications.md`, for what the engines were meant to
  become and what of it is not built.
- `README.md` (what the project promises its users), `CLAUDE.md` and
  everything under `.claude/rules/`.
- The whole of `core/src`, `cli/src` and `bench/src`, the tests, the
  flake's modules, `bench/RESULTS.md` and the result files behind it.

## Goal

After sixteen steps the suite has its engines, its checker, proof nets,
interactive proving, four exports, certificates, parallel search and two
measured baselines. Before anything is added, say where it stands and
what should come next: an honest account of the repository as it is, a
reading of what the baselines show, an assessment of every candidate in
`plan/later.md` (and of any you find missing), how they depend on and
interfere with each other, and the order you would take them in. Then put
the decisions that are the author's to the author, and plan the steps
from 18 on as the author decided: prompts in the style of the plan's
others, the step table, the models and efforts.

The project is its author's tool for research and teaching in linear
logic. Judge value by that: what a logician proving sequents, drawing
nets, checking proofs and showing them to students gains, and what a
reader of a paper that cites the tool would expect of it.

## Part one: the assessment

1. **The state of the repository.** What exists against what `README.md`,
   `CLAUDE.md`, the rules files and the plan say exists: every claim that
   is stale, every feature that is there but undocumented, every
   limitation a user hits that nothing mentions. The quality of the code
   as a maintainer finds it: where modules have grown past what one
   reader holds in their head, where two pieces do the same thing, where
   an API is inconsistent with its neighbours, where an invariant lives
   only in a rules file and nothing checks it, where tests are missing
   for a stated behaviour or present in excess of what they pin, where
   the crate-wide lint allowances in `core/src/lib.rs` hide something.
   This feeds the "Code audit and refactoring" candidate: give it a
   concrete list, ranked by what each item costs to leave alone. The
   follow-up lists in `plan/later.md`: which entries are done, which are
   obsolete, which still stand.
2. **What the baselines say.** Where each engine stands after the
   performance pass: what it decides, what it does not and why (the time
   limit, the copy bound, the recursion limit, a width), per family and
   per LLTP collection, sequentially and in parallel. How that compares
   with the provers LLTP itself records (the library ships result files
   of other provers; use them, and say what is and is not comparable).
   Which of the remaining losses are an engine's algorithm, which a
   limit, which the problem's nature. What the numbers say about the
   dispatch between the engines.
3. **Every candidate, one by one.** For each: what it is for and who
   would use it; what in the code it builds on and what would have to
   change first; whether its premises still hold after steps 15 and 16
   and in the world outside (check the current state of what it depends
   on: the proof assistants' libraries, the packages, the tools, the
   papers; the sketches' facts may be stale, and what the sources say
   wins); how large it is, in sessions of the kind the plan has run so
   far, and which model and effort its kind of work wants, with the
   reason; what could go wrong; and what it would be measured or checked
   by. Say plainly where a candidate is not worth doing, or not yet.
4. **How they interact.** Which candidate must precede which and why
   (what one changes under another); which compete for the same code, so
   that doing both in either order costs rework; which would be cheaper
   together than apart; which close off or open up others. The audit and
   refactoring is the obvious case: what it should settle before new
   engines are written on top, and what it should leave alone until they
   are.
5. **What is missing.** Candidates the list does not have and the state
   of the repository, the baselines or the project's purpose call for.
   Propose them in the same form.
6. **The order you propose.** A roadmap from step 18: each step with a
   name, a goal in two or three sentences, its boundary (what it does not
   do), its prerequisites, the follow-up entries folded into it, the
   model and effort you would give it and why, and how its result would
   be verified. Candidates you would drop or defer are listed with the
   reason.
7. **The author's decisions.** Wherever the order, the scope or the
   dropping of a candidate depends on something only the author can
   decide (research use against teaching use, the web front end's
   priority, whether a second certificate kernel matters to them, how
   much behaviour an audit may change, which candidates they do not want
   at all), make it a question: what is being decided, the options, what
   each option does to the roadmap, and the one you recommend with its
   reason. Few questions, each one the author can answer in a line.

Write the report (`plan/reports/17-assessment.md`, with a summary of one
page at the top that the author can read alone), commit it as "Assess the
project and the work ahead", and end your turn with that summary and the
questions. This is the one place in the plan where a session stops to
ask: do not plan past the questions on assumed answers.

## Part two: the plan, once the author has answered

The author answers, and may add wishes of their own or strike candidates.
Record the answers at the end of the report as given. Then plan:

1. **The roadmap**, revised by the answers: the steps from 18 in order.
   `plan/later.md` keeps what is deferred or dropped, with the reason,
   and the follow-up lists with their entries brought up to date and
   assigned to the steps that take them.
2. **The prompts.** One file per step, `plan/NN-name.md`, in the form
   the plan's other prompts have (read `09-interactive.md`,
   `13-parallel.md` and `15-performance.md` as models): the opening
   paragraph with what to read first, what the earlier steps left, the
   goal, what to build as numbered requirements, constraints,
   verification, deliverables with the report's name. A prompt says what
   is wanted and why and leaves the design to the session; it names the
   decisions of `plan/README.md` that bind it (D15 for anything a user
   sees); it carries the shared-machine and measurement rules where the
   step measures. `plan/conduct.md` is appended to every prompt when it
   is run, so a prompt does not repeat it. Write in full the prompts of
   the steps whose ground is settled, which is at least the next two;
   for a step that depends on what an earlier one will find, write what
   can be fixed now and mark, in the step table, that its prompt is
   finished after that step's review, as the plan did for its
   performance pass.
3. **Models and efforts.** For each step, the model and effort, chosen
   as "Why these models and efforts" in `plan/README.md` chooses them:
   from the current model documentation (check it on the day; the page
   names are in that section) and from what sixteen steps showed about
   which kind of work each model did well. Add a dated paragraph there
   with your choices and reasons.
4. **The plan's own files.** The step table and the command list of
   `plan/README.md` gain the new steps; a design decision the author made
   in answering becomes a numbered decision there (D16 and on) if later
   steps must obey it; the Status log gets an entry saying what was
   assessed, what the author decided and what was planned. README's
   "planned" list follows the roadmap, without mentioning the plan.

Commit as "Plan the steps from 18". Every later step is still reviewed by
the planning session after it runs, as `plan/README.md`, "Review
protocol", describes; say in your final message which step is next and
give its command.

## Constraints

- No code changes, no refactoring, no fixes: what you find goes into the
  report and, where it is work, into a step. Besides the report you write
  only under `plan/` and README's list of what is planned. A factual
  error in a documentation file is listed and given to a step, not
  corrected here.
- You may run the tests, the checks and small measurements, by the rules
  for a shared machine in "How to work on this step" below; no baseline,
  nothing on every core.
- Claims about the code cite the file and the item; claims about the
  world outside cite the source and the date you looked. Where you could
  not verify something, say so rather than assert it.
- The sketches in `plan/later.md` are starting points, not commitments:
  a candidate may be split, merged with another, narrowed or dropped, and
  the author's answers outrank both the sketches and your proposal.
- The instructions below about adding features, tests and README
  sections are written for steps that build; for this one they apply
  only in so far as they concern how to verify and how to report.

## Deliverables

- `plan/reports/17-assessment.md`: the one-page summary; the state of the
  repository; the reading of the baselines; the assessment of every
  candidate; their interactions; the candidates proposed; the order
  proposed; the questions to the author and, after part two, the answers
  and what they changed.
- After the answers: the prompts `plan/NN-name.md`, `plan/later.md` as
  what is deferred and the follow-ups, and `plan/README.md` with the
  steps, the commands, the models and efforts and the Status entry.
- Two commits, one per part.
