# Atomic Stellar payouts

Deploy this helper once, then have your server generate a `pay` transaction through Crossmint's Stellar contract-call API. It transfers a single token directly from one source wallet to every recipient. If any transfer fails, the whole batch reverts. The helper never takes custody of the funds.

This is a customer-deployed contract sample. It is not a built-in Crossmint batch endpoint. The 32-payment guard belongs to this sample; simulation and network resource limits can require smaller batches.

## Contract and authorization

The contract in [src/lib.rs](src/lib.rs) takes a token contract address, source wallet address and a typed list of `{ to, amount }` payments. Amounts are positive integer base units.

`from.require_auth()` creates one root authorization for `pay`. Both nested token transfers use that same source. The source wallet signs the entire invocation tree, including the token, recipients and amounts. Recipient signatures are not needed.

Use a recovery signer or an unrestricted operational signer. Transfer-only permissions do not authorize this custom `pay` contract call. The sample does not handle multiple source wallets or threshold signing.

## Test and build

The manifest pins `soroban-sdk = 27.0.0` and requires Rust 1.91 or later. Keep `Cargo.lock` for reproducible dependencies. Install the Stellar CLI and `wasm32v1-none` target following the [official setup guide](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup).

The lockfile retains `ed25519-dalek` 2.2.0, matching the [Crossmint contract workspace](https://github.com/Crossmint/stellar-smart-account/blob/main/Cargo.lock). An unconstrained dependency refresh can select its incompatible 3.x release through Soroban 27's broad dependency range.

Run from this directory:

```sh
cargo test --locked
cargo fmt --check
stellar contract build
```

The build writes `target/wasm32v1-none/release/atomic_payouts.wasm`. The build command and artifact path follow the [official contract build instructions](https://developers.stellar.org/docs/build/smart-contracts/example-contracts/workspace).

The native tests in [src/test.rs](src/test.rs) cover:

- Two successful payments and no balance held by the helper.
- One exact source authorization containing the root call and both transfers.
- A failure on the second payment reverting the first payment.
- Missing source authorization rejecting without moving funds.

The tests use the Soroban host and a test Stellar Asset Contract. They exercise contract execution and authorization trees with mocked signatures, not Crossmint OTP/device cryptography or a deployed network.

Validation completed with Rust 1.91.0: all four native tests passed, `cargo fmt --check` passed, and `cargo build --locked --target wasm32v1-none --release` produced the contract Wasm. The typed JSON request was also encoded against the compiled Wasm ABI, preserving both `i128` amounts and `Address` recipients. No contract was deployed and no network transaction was submitted. The recommended Stellar CLI build command above was documented from official instructions; it was not run locally.

## Deploy to testnet

Use an existing funded testnet deployment identity. These commands deploy a contract; they are instructions for your integration and were not run when preparing this sample.

```sh
stellar contract deploy \
  --wasm target/wasm32v1-none/release/atomic_payouts.wasm \
  --source-account YOUR_TESTNET_DEPLOYER \
  --network testnet \
  --alias atomic_payouts
```

Save the returned `C...` contract ID as `BATCH_PAYMENTS_CONTRACT_ID`. The flags follow the [official testnet deployment guide](https://developers.stellar.org/docs/build/smart-contracts/getting-started/deploy-to-testnet). The helper, source wallet and token must all be on the same network.

## Generate the transaction on your server

This uses the Python `requests` pattern from [USDM1's server](https://github.com/Crossmint/usdm1-wallets-expo-demo/blob/main/server/crossmint_client.py). Copy [create_transaction.py](create_transaction.py) beside `crossmint_client.py` so it uses your existing API URL and server key. Keep the API URL and credentials on the same network as the deployed contract.

Assumption: the source is an ordinary deployed Crossmint Stellar smart wallet, and the external signer is already registered on it without transfer-only scopes. Owning a private key alone does not register it as a wallet signer. This sample does not assume a special treasury-wallet API configuration.

```python
from create_transaction import create_batch_transaction

transaction = create_batch_transaction(
    wallet_address=SOURCE_WALLET_ADDRESS,
    signer_locator=f"external-wallet:{REGISTERED_SIGNER_PUBLIC_KEY}",
    batch_contract_id=BATCH_PAYMENTS_CONTRACT_ID,
    token_contract_id=TOKEN_CONTRACT_ID,
    payments=[
        {"to": RECIPIENT_A, "amount": "10000000"},
        {"to": RECIPIENT_B, "amount": "25000000"},
    ],
)
```

The function sends one `POST /api/2025-06-09/wallets/{walletAddress}/transactions` with `params.signer` and `params.transaction.type: "contract-call"`. It returns the transaction, including its `id` and `approvals.pending`. Signing and approval remain with your process; transaction generation does not broadcast the payouts.

For a seven-decimal token, these amounts are 1 and 2.5 tokens. Use the token's Soroban/SAC contract address. Keep amounts as decimal strings; the typed contract ABI encodes them as `i128` and recipients as `Address`.

The current token transfer API generates one transaction per recipient. It cannot turn a list of transfer requests into an atomic batch. This sample implements atomicity inside one custom contract invocation; no generic public `multicall` transaction type or serialized-XDR submission is assumed.

The source needs sufficient spendable tokens, and recipients must satisfy the token's authorization rules. Simulation and network resource limits still apply. Reconcile the original transaction before retrying an uncertain result, since atomicity does not prevent duplicate batches.

The contract execution and JSON ABI encoding were tested locally. The Python request matches the deployed public API schema and assembly path, but no Crossmint network transaction was generated or executed during preparation. Validate the deployed contract/wallet/token combination on staging before production use.

## Why the helper is typed

The helper's `Vec<Payment>` preserves `Address` and `i128` types through JSON and the on-chain ABI. A generic dispatcher's `Vec<Val>` does not supply these inner types, so a JSON array of address/base64 strings is not an equivalent request. The [Stellar ABI encoder](https://github.com/stellar/js-stellar-sdk/blob/v17.0.1/src/contract/spec.ts) shows how typed struct fields are encoded.
