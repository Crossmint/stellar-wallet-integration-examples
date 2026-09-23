# Repository instructions

This repository contains reusable Crossmint Stellar wallet examples. Start with [README.md](README.md), then the [atomic payout guide](samples/atomic-payouts/README.md) for the implementation details.

## Scope

- Keep examples independent of customers, app frameworks and private repositories.
- Transaction generation runs on the server through REST. Approval belongs to the registered signer; do not replace this with a client-side transaction-generation flow.
- The payout helper is Soroban-specific: one source, one token, multiple recipients. Preserve its typed ABI, root source authorization and failure propagation.
- Keep the caller-provided idempotency key mandatory. It belongs to a persisted payout job, stays unchanged on retries and must not be generated inside the request helper.
- A configurable token is a trust boundary. Root authorization binds the token and arguments; nested actions requiring source authorization must also be approved. Keep the untrusted-token regression tests and integration guidance.
- Never commit API keys, private keys, wallet exports, customer data, virtual environments or build artifacts.

## Verification

Run from the repository root, with Python dependencies and the Rust toolchain installed:

```sh
python -m unittest discover -s samples/atomic-payouts -p 'test_*.py' -v
cargo test --locked --manifest-path samples/atomic-payouts/Cargo.toml
cargo fmt --check --manifest-path samples/atomic-payouts/Cargo.toml
cargo build --locked --target wasm32v1-none --release --manifest-path samples/atomic-payouts/Cargo.toml
git diff --check
```

For contract ABI changes, verify the JSON arguments still encode against the compiled Wasm specification. Preserve decimal-string amounts and address types.

## Documentation and delivery

- Keep the README focused on setup, usage and verification. Customer message drafts do not belong in this repository.
- Keep the AI-generated proof-of-concept notice and separate local tests from deployed Crossmint results. Mocked authorization does not prove signer cryptography or end-to-end execution.
- Keep examples and documentation synchronized when function signatures or requirements change.
- Use a branch and PR for changes. Report the checks run and any untested network behavior; do not infer production readiness from passing local tests.
