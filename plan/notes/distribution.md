# Releases, registries, repositories and policies

Facts checked on 2026-10-03 by a sub-agent of step 17, from the policy
pages and repositories named; "raw" marks what was read in the source
itself and not through a summary. Step 31 reads this before it prepares
the release; D22 of `plan/README.md` is the decision that rests on it.
Every outward act (a transfer, a tag, a publication, a pull request
elsewhere) is the author's.

## One repository or several

A library, its command and its web front end in one Cargo workspace is
the norm (raw: ripgrep, typst, rust-analyzer, tokio, serde, bevy, leptos,
z3.rs each keep `crates/*` in one repository). What such projects split
off is content: a website, a book, a package index. Provers go both ways
for their proof-assistant part: NanoYalla is a directory of Click &
coLLecT, Yalla and coq-elpi have repositories of their own.

So: `core/`, `cli/`, `bench/` and the web front end's bindings (the
crate `linlog-web`) stay one workspace, since they version and test
together and the bindings are written against the library's types. The
web front end's client is a repository of its own from the start
(decided by the author on 2026-10-03): it has another toolchain, it
redeploys on every push where the library releases by version, and as
`linlog-prover.github.io` it is served at the organization's root
address while this repository's Pages keep the rustdoc. It takes this
repository as a pinned flake input, so a change across both is two
commits and nothing breaks unannounced. The precedents: rust-analyzer
keeps its editor client in its tree because the two share a protocol;
tokio, bevy and egglog keep sites and demos apart. The Rocq library starts under `rocq/`
(D20) and moves out when it has a release rhythm or contributors of its
own; the opam archive needs only a release archive. What may leave the
history is data: the baselines' CSV files (11 MB after two) fit release
assets or Zenodo better. `plan/` stays.

## A GitHub organization

Free organizations have unlimited public repositories, Actions on
standard runners and Pages (a site of at most 1 GB). A transfer keeps
issues, stars and secrets and redirects the web and git URLs; Pages is
not redirected, and whether Nix follows the redirect of
`github:flgrubm/linlog` is inferred, not verified. The name `linlog` is
taken on GitHub (a user account of 2018; raw); `linlog-rs`, `linlog-org`
and `linlog-prover` were free. The owner is decided before the first
publication, because the repository's URL goes into crate metadata that
cannot be changed, into the Pages address and into Zenodo's records.

**Decided (the author, 2026-10-03): the organization `linlog-prover`.**
The name was free on that day (`api.github.com/users/linlog-prover`
answered 404). An organization on github.com is created in the browser
by the account that will own it (github.com/account/organizations/new,
the Free plan); no session can do it. Creating it at once holds the
name; the repository can follow at any quiet moment before step 31
(Settings, Danger Zone, Transfer). What a transfer touches, for the
session that prepares the release to bring in line:

- the remote: `jj git remote set-url origin
  https://github.com/linlog-prover/linlog` (the old address redirects,
  so nothing breaks before that);
- CLAUDE.md, which names `github.com/flgrubm/linlog` as `origin`, and
  README, which links the documentation site: Pages moves to
  `linlog-prover.github.io/linlog` and is not redirected, and the
  repository's description and homepage on GitHub point at the old one;
- the manifests' `repository` and `homepage`, the `CITATION.cff`, the
  opam file's `homepage`, `dev-repo` and `bug-reports`, which are all
  written for the first time by step 31 and step 28 and take the new
  address from the start;
- Trusted Publishing and Zenodo, which are configured per owner and are
  set up after the transfer, not before.

## crates.io

- `linlog`, `linlog-cli`, `linlog-bench` and `linlog-web` are free (raw).
  A crate that "exists only to reserve a name" is against the policy, so
  no placeholder.
- EUPL-1.2 is a valid SPDX identifier there. The policy says nothing on
  AI.
- `cargo publish --workspace` is stable since Cargo 1.90. Missing in the
  manifests today: `repository`, `readme`, keywords and categories, a
  licence file inside `core/` and `cli/`, a description of its own for
  `linlog-cli`, and `[package.metadata.docs.rs]` with all features so
  that `parallel` is documented.
- Trusted Publishing: the first publication needs a token; after it the
  crate's settings name the repository and workflow, and the workflow
  publishes with a short-lived token (`rust-lang/crates-io-auth-action`).
  release-plz, cargo-release and cargo-dist are all maintained;
  release-plz does the release pull request, the changelog, the tags and
  the publication, cargo-dist only matters if binaries are asked for.
- `CITATION.cff` on the default branch gives "Cite this repository";
  Zenodo, switched on for the repository, archives every GitHub release
  with a DOI.

## The Rocq library

A pull request to `rocq-prover/opam` adds
`released/packages/rocq-linlog/rocq-linlog.X.Y.Z/opam` (the name starts
with `rocq-`; a release archive with its checksum; the tag `logpath:`);
`opam lint --check-upstream` first. No `rocq-linlog` or `coq-linlog`
exists (raw). In nixpkgs a Rocq library is `mkRocqDerivation` under
`rocqPackages`, which is the shape step 28's flake package follows. The
Rocq Platform asks for evidence of use and is far off.

## nixpkgs

A new package lives at `pkgs/by-name/li/linlog/package.nix` with a
maintainer; `lib.licenses.eupl12` exists; a tagged release is the normal
case. The project "too immature" or "a niche of 5 people" is not wanted:
wait for users beyond the author.

## Policies on AI assistance

- **crates.io, docs.rs, GitHub, the Rocq opam archive, Rocq's
  contribution guide**: no statement (raw for each but docs.rs). GitHub:
  "You are responsible for Your Content".
- **nixpkgs** (raw, CONTRIBUTING.md): a responsible person in the loop;
  a commit to nixpkgs written with an LLM carries an `Assisted-by:`
  trailer naming tool and model; "use of automated tools to develop
  upstream software packaged inside Nixpkgs is not in scope". So
  linlog's own history is no obstacle, and a packaging pull request
  written with Claude needs the trailer.
- **Zenodo**: AI tools are not authors; substantive use is disclosed in
  the description.
- **Venues** (ACM, Dagstuhl's LIPIcs, Springer): generative AI may be
  used, is disclosed in the work, and is never an author.
- **The EUPL**: the Commission's FAQ says nothing on AI-assisted code;
  no official guidance was found.

What follows for linlog: nothing forbids any of the publications. The
README and the Zenodo description say how the code was written.

## Threads and time limits elsewhere

- **Provers default to one thread**: Z3 (`parallel.enable` false), cvc5,
  Vampire (`cores` 1), E, Kissat, CaDiCaL (raw for Vampire, E, Kissat,
  CaDiCaL); Click & coLLecT's prover is sequential.
- **Time limits by default** exist where a person waits: Click &
  coLLecT 3 s, Sledgehammer 30 s, Vampire 60 s (raw); Z3, cvc5, E and
  the SAT solvers have none.
- **Command-line tools default to every core, capped**, and stay on one
  thread by a cheap static test: ripgrep one thread for a single file,
  else at most 12; fd at most 64; GNU sort at most 8 and no threads
  below 128 K lines (raw).
- **Sequential first, then parallel** has precedents in thresholds
  (rayon's `with_min_len`, Java's guidance for parallel streams,
  heartbeat scheduling), not in any prover's default. One thread for a
  short budget and then the pool, which step 21 builds, is linlog's own
  design in that spirit; its budget is an option (D16).
