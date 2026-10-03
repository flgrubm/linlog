# Step 27: the web front end

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes three
sessions and has a plan of its own, which its first session writes; this
prompt is finished at the reviews of steps 22 to 24. Read before you
start:

- `plan/later.md`: "The web front end", "Follow-ups: interactive
  proving".
- `plan/reports/09-interactive.md` ("For steps 10 to 12 and the web
  front end"), `11-svg.md` ("What the web front end will call"),
  `17-assessment.md` (3.14, D9, the author's answers), `18`, `22`, `23`.
- `plan/README.md`: D11, D12, D13, D15, D16, D18.
- `plan/notes/distribution.md`.

## Goal

A student opens a page, types a sequent and proves it by clicking, or
lets the search close a goal, and sees the derivation and, for MLL, the
net drawn as the command draws them. Nothing is installed and nothing is
sent to a server.

## What is fixed now

1. **Two repositories** (D22; the author, 2026-10-03). In this
   workspace the crate `linlog-web`, the bindings: the library compiled
   to `wasm32-unknown-unknown` without the `parallel` feature, behind a
   small API of JSON in and JSON or SVG out (the interactive state, the
   options values of steps 22 and 23, the outcome of a search), with its
   tests run under wasm and its build a flake check from the first
   session, so that a change of the library that breaks them fails here.
   In a repository of its own under the organization `linlog-prover`
   the client: the page, its interface code, its assets and its
   deployment, with a flake that takes this repository as a pinned
   input, its own CI, licence headers and CLAUDE.md. The session sets
   that repository up locally, with jj as this one is, in the directory
   the author names; creating it on GitHub and pushing are the author's.
   Named `linlog-prover.github.io` it is served at the organization's
   root address, with the rustdoc staying at `/linlog`; propose that
   and let the author decide.
2. **What the target lacks** (checked 2026-10-03): `std::thread::spawn`
   and `Instant::now()` panic there; the stack is 1 MiB unless a link
   argument raises it; threads need cross-origin isolation, which GitHub
   Pages cannot set. So: one thread; a stop that counts the engine's
   work, with a budget that is an option; the stack raised or the
   recursion limit lowered, with a test at the limit under wasm; step
   18's bound on every derivation (`ViewOptions`), with a smaller
   default than the command's, and `Size::height` asked before a
   derivation is built, since the builder recurses to that depth unless
   step 20 gave it a stack of its own (it did: the builder, the parser,
   the printers and sequentialization no longer recurse on the input).
   The target is the first 32-bit one: the arguments that no integer of
   the checker, of the size estimate and of the memo wraps were made for
   64 bits, with a sentence each for 32, and nothing was ever compiled
   there. The bindings' tests run the checker's and the memo's tests at
   their limits under wasm32, and the memory and occurrence bounds get
   defaults that fit a tab (the library's gibibyte is close to what a
   tab has in all).
3. **The two searches of the default bias** take turns from their start
   without threads, at up to five times the better search. The first
   session measures it under wasm on the target set's small rows and
   says whether a search that can be suspended (an explicit stack in the
   focused engine) is needed; if so it is step 29's, not this step's.
4. **The first version is `linlog interact` with a mouse** and no more:
   a sequent, the modes, the goals with clickable formulas, the rules
   that apply, undo, close, the drawing, the finished proof's exports.
   Its plan says what is left out.
5. **Hosting** from the client's repository, built by its flake; this
   repository's Pages keep the rustdoc.

## What waits

The choice of client technology (the first session compares a plain page
over wasm-bindgen with one Rust framework, by size, maintenance and what
the author would read); the names of steps 22 and 23. One caveat on the
two repositories: a client written in a Rust framework is a crate that
depends on `linlog`, which is the case for a workspace. If the
comparison ends there and the split then costs more than it gives, say
so with the reasons and ask the author before departing from D22.

## Deliverables

`plan/web/README.md` (the step's own plan, by its first session) and
`plan/reports/27-web.md`, both in this repository; thematic jj commits
here for the bindings and in the client's repository for the client.
