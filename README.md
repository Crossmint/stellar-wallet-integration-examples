# Stellar wallet integration examples

Generate atomic token payouts from a Crossmint Stellar smart wallet using a server-side REST request. The included Soroban contract transfers one token from one source wallet to multiple recipients in one invocation. If a transfer fails, the group reverts.

**AI-generated proof of concept.** Review the contract and integration code carefully. Local tests pass, but this sample has not been independently audited or tested end to end through Crossmint. Validate on staging/testnet before using real funds.

## How it works

1. Deploy the Soroban helper once on the wallet's network.
2. Persist a payout job, its payment list and a unique idempotency key on the server.
3. Call `create_batch_transaction` to generate one `pay` transaction through Crossmint.
4. Have the wallet's registered signer approve the returned transaction through the application's signing flow.
5. Verify the final transaction status and recipient balances before marking the job complete.

The Python function only generates the transaction. Signing, job persistence and reconciliation belong to the integrating application. The implementation targets Stellar; the batching concept is similar to [Safe MultiSend](https://github.com/safe-fndn/safe-smart-account/blob/main/contracts/libraries/MultiSend.sol) on EVM.

## Prerequisites

- Python 3.10 or later, Rust 1.91 or later, the `wasm32v1-none` target and the [Stellar CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup).
- A Crossmint staging server API key with `wallets:transactions.create` permission.
- A deployed Crossmint Stellar smart wallet with a registered, unrestricted external signer. Possessing a private key alone does not register that signer on the wallet.
- A funded testnet deployment identity, a trusted token contract and funded source wallet. Recipients must satisfy that token's authorization requirements.

## Set up and test

Clone the repository, then run the following from `samples/atomic-payouts`:

```sh
cd samples/atomic-payouts
python3 -m venv .venv
. .venv/bin/activate
python -m pip install requests
python -m unittest discover -p 'test_*.py' -v
cargo test --locked
cargo fmt --check
stellar contract build
```

Keep `Cargo.lock`. See the [sample guide](samples/atomic-payouts/README.md#test-and-build) for dependency constraints and what the tests establish.

## Deploy the helper

Use a funded testnet identity configured in the Stellar CLI:

```sh
stellar contract deploy \
  --wasm target/wasm32v1-none/release/atomic_payouts.wasm \
  --source-account YOUR_TESTNET_DEPLOYER \
  --network testnet \
  --alias atomic_payouts
```

Save the returned contract ID as `BATCH_PAYMENTS_CONTRACT_ID`. The helper, source wallet and token must use the same network. Preparing this repository did not deploy a contract.

## Generate a batch on the server

Load these values from server configuration or the persisted payout job:

| Value | Meaning |
|---|---|
| `CROSSMINT_API_URL` | `https://staging.crossmint.com/api/2025-06-09` for staging |
| `CROSSMINT_API_KEY` | Staging server key; keep it out of client code and Git |
| `SOURCE_WALLET_ADDRESS` | Deployed Crossmint Stellar wallet |
| `REGISTERED_SIGNER_PUBLIC_KEY` | External signer's public key registered on that wallet |
| `BATCH_PAYMENTS_CONTRACT_ID` | ID returned by deployment |
| `TOKEN_CONTRACT_ID` | Trusted token contract selected from server configuration |
| `BATCH_IDEMPOTENCY_KEY` | Unique key persisted for this logical payout job |
| `PAYMENTS_FILE` | Path to a JSON array of payments, as shown below |

Create the payments file using real testnet recipients and integer base-unit amounts represented as strings:

```json
[
  { "to": "<RECIPIENT_A>", "amount": "10000000" },
  { "to": "<RECIPIENT_B>", "amount": "25000000" }
]
```

From the same sample directory, generate the transaction:

```python
import json
import os
from pathlib import Path

from create_transaction import create_batch_transaction

transaction = create_batch_transaction(
    api_url=os.environ["CROSSMINT_API_URL"],
    api_key=os.environ["CROSSMINT_API_KEY"],
    idempotency_key=os.environ["BATCH_IDEMPOTENCY_KEY"],
    wallet_address=os.environ["SOURCE_WALLET_ADDRESS"],
    signer_locator=f"external-wallet:{os.environ['REGISTERED_SIGNER_PUBLIC_KEY']}",
    batch_contract_id=os.environ["BATCH_PAYMENTS_CONTRACT_ID"],
    token_contract_id=os.environ["TOKEN_CONTRACT_ID"],
    payments=json.loads(Path(os.environ["PAYMENTS_FILE"]).read_text()),
)
print(json.dumps(transaction, indent=2))
```

Persist the returned transaction ID and pending approvals with the job. After a timeout, retry with the **same key and unchanged payload**. Use a new key only for a genuinely new payout job, even if its payments happen to match. Do not rotate the key to work around an uncertain response.

## Approve and verify

Use the selected signer's approval flow for the returned transaction. Inspect the complete authorization tree, including the helper, token, recipients, amounts and nested calls. Once approved, Crossmint handles execution. Reconcile the final status and recipient balances; an `awaiting-approval` response is not a completed payout.

For staging validation, run a successful batch and a batch whose later transfer fails. Confirm all expected credits in the successful case and no partial credits in the failed case. Local host tests cover rollback, but the deployed Crossmint flow still needs this validation.

## Constraints and references

- One source wallet and one token per invocation. The sample caps a batch at 32 payments; simulation and network limits may require fewer.
- Transfer-only signer scopes do not authorize the custom `pay` call. This sample assumes an unrestricted signer.
- The helper accepts a token address. Select it from a trusted per-network configuration or allowlist, not unvalidated client input. An approved malicious token can request unintended nested calls. See the [authorization boundary](samples/atomic-payouts/README.md#contract-and-authorization).
- Atomic execution does not prevent duplicate payouts submitted as separate transactions; the persisted idempotency key addresses creation retries.
- [Detailed contract and server guide](samples/atomic-payouts/README.md), [contract](samples/atomic-payouts/src/lib.rs), [Rust tests](samples/atomic-payouts/src/test.rs), [Python tests](samples/atomic-payouts/test_create_transaction.py).
- [Recovery, operational permissions and WhatsApp reference](docs/wallet-signers.md).
- AI contributors: read [AGENTS.md](AGENTS.md) before changing the examples.
