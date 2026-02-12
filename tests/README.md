# Integration Test Layout

This directory is reserved for black-box Rust integration tests.

Conventions:

- Put tests here only when they can use public crate interfaces.
- Keep module-internal unit tests colocated under `src/<module>/tests.rs`.
- Use Make targets (`make test`, `make test-unit`, `make test-integration`) as the official execution flow.
