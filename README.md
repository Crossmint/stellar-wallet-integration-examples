# Stellar wallet integration examples

These changes follow the [USDM1 demo architecture](https://github.com/Crossmint/usdm1-wallets-expo-demo): Python creates transactions; the app approves them. Wallet creation, authentication and ordinary approval flows are unchanged.

## Multiple recovery methods

Use our [recovery POC](https://github.com/Crossmint/crossmint-stellar-wallets-demo/pull/5) as the implementation reference. Its [server functions](https://github.com/Crossmint/crossmint-stellar-wallets-demo/blob/main/lib/crossmint-server.ts) and [approval hook](https://github.com/Crossmint/crossmint-stellar-wallets-demo/blob/main/hooks/use-signers.ts) show the server-request/client-approval split.

The new fields are `config.recoveryMethods` for creation and `approver` when choosing which recovery method authorizes a signer change. Your transfer requests already carry `signer`, which also disambiguates multi-recovery wallets. The POC contains the complete flow, so this repository does not duplicate it.

## Operational signer permissions

Merge these fields into the existing `POST /wallets/{walletLocator}/signers` body alongside `signer` and `approver`:

```json
{
  "scopes": [{
    "type": "transfer",
    "tokenLocator": "stellar:usdc",
    "spendingLimit": { "amount": "100", "interval": 86400 },
    "recipients": ["<COMPANY_WALLET_ADDRESS>"]
  }],
  "expiresAt": "2026-12-31T23:59:59Z"
}
```

This grants 100 USDC per 86,400-second interval to that recipient. `expiresAt` applies to the signer. It does not grant signer administration. The registration response and existing approval flow are unchanged.

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

Then run your existing approval/OTP flow. Reapply `channel` when selecting the signer after loading the wallet. It is a client setting; it is not a new REST field or a different phone identity. The published wallets SDK 1.17.0 supports it. The USDM1 reference repo pins an older React Native SDK, so use a current React Native package containing this wallet SDK when adopting the new features.

## Batch transaction generation on your server

The token transfer endpoint generates one transaction per recipient. It does not accept an atomic list of recipients. Multiple calls, including parallel calls, remain separate transactions.

For a single atomic payout, deploy the included [typed batch contract](samples/atomic-payouts/src/lib.rs). Your server then generates one ordinary `contract-call` transaction to `pay`, using the registered external signer locator. The [Python generation function](samples/atomic-payouts/create_transaction.py) uses the same `requests` pattern and API constants as your existing `server/crossmint_client.py`; no client SDK creates the transaction.

[Contract setup and server request](samples/atomic-payouts/README.md)

The example assumes a deployed Crossmint Stellar smart wallet, one source, one token and a registered unrestricted external signer. It is a custom-contract integration, not a built-in batch-transfer endpoint. The contract must exist on the same network before Crossmint can inspect its ABI and simulate the transaction. The API returns a transaction awaiting the selected signer's approval.

## Repository layout

- `samples/atomic-payouts/README.md`: build, deployment and transaction-generation instructions
- `samples/atomic-payouts/create_transaction.py`: Python server request
- `samples/atomic-payouts/src/`: Soroban contract and four execution/authorization tests
- `samples/atomic-payouts/Cargo.toml` and `Cargo.lock`: reproducible Rust dependencies
