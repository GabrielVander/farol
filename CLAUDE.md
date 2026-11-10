# CLAUDE.md

Engineering rules for this repository. They are principles, not a map of the
code: crates, modules and types get renamed and added, so don't rely on names.

**Before changing anything:** find the current crates in the workspace
`Cargo.toml`, read each affected crate's `Cargo.toml` and module tree, and
follow the patterns already in the code.

## Clean Architecture with Vertical Slices

- Layers, from inside out: **Entities → Use Cases → Adapters → Infrastructure**.
  Use cases define the gateway traits they need.
- Dependencies only point inward. The core crate never depends on a slice or an
  app crate.
- Crate roles:
  - **Core:** one crate with the entities, use cases and gateway traits. No
    provider-specific code, no HTTP and no serialization.
  - **Slices:** one crate per DNS provider or public IP source. They implement
    the core gateways.
  - **Apps:** composition roots. They wire up use cases and gateways and hold
    no business rules.
- A new DNS provider or IP source means a new slice crate, not new code in
  the core or an app.
- I/O is blocking. Don't add an async runtime: farol runs once and exits.

## Domain-Driven Design (lightweight)

- **Ubiquitous language:** use the domain terms the core entities already use,
  the same way in code, tests and discussion. Don't invent synonyms. When a
  concept is renamed, rename it everywhere.
- **Bounded contexts:** each slice is its own context with its own model (for
  example, a provider's zone and record DTOs). Slice-internal concepts never
  appear in the core's API.
- **Anti-corruption layer:** a slice translates its model into core entities
  inside its adapters, and nowhere else.
- **Value objects:** immutable, compared by value, valid by construction. Use
  newtypes, private fields and validating constructors (`TryFrom` / `new`
  returning `Result`) so invalid states can't be represented.
- **Not used**, unless a real need shows up and the user agrees: aggregates,
  repositories, domain events, domain services, CQRS.

## Clean Code (Robert C. Martin), applied faithfully

### Functions

- **Small.** Target about 4–5 lines and never go over 20. **Blank lines don't
  count** toward the length.
- Do one thing. One level of abstraction per function. Code reads top-down
  (the stepdown rule).
- Arguments: 0–2 is ideal, 3 needs a justification, more is not allowed. No
  flag (boolean) arguments; split the function instead.
- No hidden side effects. Keep command–query separation.
- Prefer `Result` to error codes. Don't panic (`unwrap`, `expect`, `panic!`)
  in library code; it's acceptable in tests.

### Names

- Reveal intent, can be pronounced and searched, no encodings or type
  prefixes.
- Use nouns for types and values, verbs for functions. Use one word per
  concept across the codebase.

### Comments

A comment is an admission that the code didn't explain itself. Before writing
one, try renaming, extracting a function or adding an explanatory variable.

Only these comments are allowed:
- Legal or licence headers.
- Informative comments that code can't express (for example, the format a
  pattern matches).
- Explanation of intent: *why*, never *what*.
- Clarifying an obscure argument or return value from code you can't change
  (std or third-party APIs).
- Warning of consequences (for example, a slow or non-thread-safe test).
- `TODO`, used sparingly and with a reason.
- Amplification of something that looks trivial but matters.
- `///` doc comments on **public** API only.

These comments are not allowed:
- Mumbling, redundant comments that restate the code, misleading comments.
- Mandated per-function or per-field docs, noise comments
  (`/// Creates a new X`).
- Journal or changelog comments, attributions, bylines (git keeps history).
- Position markers or banners (`// ---- Helpers ----`), closing-brace comments.
- **Commented-out code: delete it.**
- HTML in comments, nonlocal information, too much information or history,
  comments with an inobvious connection to the code.
- Function-header comments on small, well-named functions. Doc comments on
  non-public items.
- A comment used where a function or variable name would do.

Keep any remaining comment accurate. When the code changes, update or delete
the comment in the same change.

### General

- DRY. Follow SOLID, with small, cohesive types (SRP) and the Open/Closed
  principle.
- Follow the Boy Scout Rule: leave the code cleaner than you found it.
- Keep the code `cargo fmt`-formatted and `cargo clippy`-clean.

## Ubiquitous language

Use these terms, and only these, in code, tests and discussion:

- **Public IP:** the IPv4 address the internet sees for this machine. Never
  private, loopback, link-local or unspecified.
- **Domain:** the DNS zone, for example `example.com`.
- **Record name:** a name relative to the domain: `@` for the apex, labels
  such as `home`, or a wildcard (`*` as the leftmost label, as in `*.home`).
- **TTL:** a record's time to live, in seconds, never zero.
- **Tracked record:** a record farol keeps pointed at the public IP: a domain,
  a record name and a TTL.
- **Drifted:** a tracked record whose addresses are anything other than
  exactly the public IP (missing, different or several).
- **Point:** make a tracked record resolve to the public IP, replacing its
  addresses.
- **Check:** find which tracked records have drifted without changing them.
- **Sync:** check, then point every drifted record.
- **Outcome:** what a check or sync found for one tracked record: unchanged,
  drifted (check only), pointed (sync only) or failed.
- **Report:** the outcome of every tracked record in one check or sync.

## Canon TDD (Kent Beck)

Follow these steps strictly, in order:

1. **Write a test list.** Before any code, write down the behaviour scenarios
   the change has to cover. This step is mandatory; don't skip it. Show the
   list before starting.
2. Turn **exactly one** item into a concrete, runnable test. Run it and watch
   it fail for the expected reason.
3. Change the code to make that test and every earlier test pass. Don't add
   anything the test doesn't need, and don't refactor yet.
4. Optionally refactor while the tests are green, without changing behaviour.
5. Go back to step 2 until the list is empty. Add scenarios you discover along
   the way to the list instead of chasing them right away.

Don't copy computed values into assertions. Don't mix refactoring with making
a test pass. Don't turn several list items into tests at once.

## Preparatory Refactoring

"Make the change easy (warning: this may be hard), then make the easy change."

- Before a behaviour change, restructure the code under green tests so the
  change becomes simple.
- The refactoring and the behaviour change are separate steps, ideally
  separate commits.

## Commits

- Make granular commits: one logical change each. A refactoring and a
  behaviour change are separate commits.
- The message is a single line, `(<context>) <Imperative summary>`, and
  nothing more: no body and no trailers (no `Co-Authored-By` or similar).
- `<context>` is the crate or area the change touches (for example `core`,
  `hostinger`, `ipify`, `cli`, `repo`).
- Example: `(core) Add sync records use case`.

## Test Coverage: aim for 100%

- Measure with `cargo llvm-cov --workspace` (add `--html` for a report).
- New or changed code should be fully covered. Explain any line that really
  can't be tested (for example, thin `main` wiring).
- Coverage comes from TDD. Don't write tests after the fact just to hit the
  number.
- Unit tests live in the module, under `#[cfg(test)]`. Use hand-written fakes
  for gateway traits (no mocking frameworks) and reuse the test helpers
  already in the code.

## Commands

All workspace-wide, so they keep working as crates are added:

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all
cargo llvm-cov --workspace
```
