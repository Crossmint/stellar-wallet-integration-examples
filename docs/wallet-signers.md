# Wallet signer configuration references

These references cover recovery methods, operational signer permissions and WhatsApp OTP. They are separate from the atomic payout contract.

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
