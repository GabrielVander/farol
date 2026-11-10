# Engineering Principles

- **Commits:** Commit as you go, in small granular steps. The message is exactly `(<context>) <imperative summary>` and nothing else: no body, no trailers, no attribution.
- **Clean Code:** Follow Robert C. Martin's *Clean Code* faithfully, especially Chapter 17, "Smells and Heuristics".
- **Clean Architecture:** Follow Robert C. Martin's *Clean Architecture* faithfully, organized in vertical slices by feature.
- **DDD:** Use Eric Evans's Domain-Driven Design lightly: ubiquitous language and bounded contexts, with tactical patterns only where they earn their keep.
- **TDD:** Apply Kent Beck's Canon TDD. Never skip step 1: write the test list before any test. Then turn one item into a runnable test, make it pass, optionally refactor, and repeat until the list is empty.
- **Preparatory Refactoring:** Apply Kent Beck's "make the change easy (warning: this may be hard), then make the easy change". Refactor under green tests first, and commit the refactoring separately from the behavior change.
- **Rust:** When writing Rust, follow Rust best practices: idiomatic code, the Rust API Guidelines, and clean `cargo fmt` and `cargo clippy`.
