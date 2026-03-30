# Contributing

## Guidelines

- One feature at a time
- Tests first or alongside, never after
- `pub(crate)` by default, `pub` only at crate boundaries
- Clippy must pass with no warnings
- BDD scenarios for domain behavior changes

## Definition of done

- Unit tests
- BDD scenarios if it touches domain behavior
- Clippy passes
- Format check passes

## Architecture constraints

- Handlers do not manage transactions — `TransactionBehavior` handles begin/commit/rollback
- SQL enforces tenant isolation — no load-then-check patterns
- Read stores are separate from write stores (CQRS)
- Domain events go through the transactional outbox

## License

AGPL-3.0. By contributing, you agree that your contributions will be licensed under the same license.
