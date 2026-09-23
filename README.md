# Stellar wallet integration examples

Reusable examples for Crossmint Stellar smart wallets: recovery methods, operational signer permissions, WhatsApp OTP and atomic payouts. Servers generate transactions through REST; the selected signer approves them through the application's signing flow. The examples do not depend on a particular customer, app framework or server layout.

**AI-generated proof of concept.** Review the code carefully and validate it on staging/testnet before using real funds. The payout contract passed local execution and authorization tests, but has not been independently audited or tested end to end through Crossmint on a deployed network.

The payout implementation is Stellar-specific. It includes a Soroban contract to deploy once per network and a Python function that generates calls to it. No contract has been deployed as part of preparing this repository.

## Multiple recovery methods

Use our [recovery POC](https://github.com/Crossmint/crossmint-stellar-wallets-demo/pull/5) as the implementation reference. Its [server functions](https://github.com/Crossmint/crossmint-stellar-wallets-demo/blob/main/lib/crossmint-server.ts) and [approval hook](https://github.com/Crossmint/crossmint-stellar-wallets-demo/blob/main/hooks/use-signers.ts) show the server-request/client-approval split.

Use `config.recoveryMethods` at wallet creation and `approver` to choose which recovery method authorizes a signer change. Transaction requests use `signer` to select the approving signer. The POC contains the complete flow.

## Operational signer permissions

Include these fields in the `POST /wallets/{walletLocator}/signers` body alongside `signer` and `approver`:

```json
{
  "scopes": [{
    "type": "transfer",
    "tokenLocator": "stellar:usdc",
    "spendingLimit": { "amount": "100", "interval": 86400 },
    "recipients": ["<ALLOWED_RECIPIENT_ADDRESS>"]
  }],
  "expiresAt": "2026-12-31T23:59:59Z"
}
```

This grants 100 USDC per 86,400-second interval to that recipient. `expiresAt` applies to the signer. Operational signers cannot administer other signers. Approve the returned registration transaction through the application's signing flow.

The reset is interval-based, not a midnight reset. Omitting `interval` makes the cap non-resetting. Omitting recipients permits any recipient; omitting scopes removes the spending restrictions. To replace an existing signer's permissions, remove and re-register it with the new scope rather than assuming this POST patches existing scopes.

[Permissions guide](https://docs.crossmint.com/wallets/guides/signers/scopes)

## WhatsApp OTP

Use this configuration when selecting the installed phone recovery signer for the transaction your backend prepared:

```ts
await wallet.useSigner({
  type: "phone",
  phone: userPhone,
  channel: "whatsapp",
});
```

Then run the approval/OTP flow. Reapply `channel` when selecting the signer after loading the wallet. This client setting keeps the same phone identity. Verified against the published wallets SDK 1.17.0; use a compatible client package exposing this option.

## Server-side batch transaction generation

The token transfer endpoint generates one transaction per recipient. It does not accept an atomic list of recipients. Multiple calls, including parallel calls, remain separate transactions.

For a single atomic payout, deploy the included [typed batch contract](samples/atomic-payouts/src/lib.rs). The server then generates one `contract-call` transaction to `pay`, using the registered external signer locator. The standalone [Python generation function](samples/atomic-payouts/create_transaction.py) accepts the API URL, server key and payment details as arguments and can be adapted to any backend.

The deployed contract executes every token transfer within one invocation. A failed transfer reverts the entire group. This is conceptually similar to an EVM batching contract such as [Safe MultiSend](https://github.com/safe-fndn/safe-smart-account/blob/main/contracts/libraries/MultiSend.sol); the implementation here uses Soroban's contract and authorization model.

[Contract setup and server request](samples/atomic-payouts/README.md)

The example assumes a deployed Crossmint Stellar smart wallet, one source, one token and a registered unrestricted external signer. It is a custom-contract integration, not a built-in batch-transfer endpoint. The contract must exist on the same network before Crossmint can inspect its ABI and simulate the transaction. The API returns a transaction awaiting the selected signer's approval.

## Repository layout

- `samples/atomic-payouts/README.md`: build, deployment and transaction-generation instructions
- `samples/atomic-payouts/create_transaction.py`: Python server request
- `samples/atomic-payouts/src/`: Soroban contract and four execution/authorization tests
- `samples/atomic-payouts/Cargo.toml` and `Cargo.lock`: reproducible Rust dependencies
