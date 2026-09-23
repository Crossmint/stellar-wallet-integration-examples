"""Generate one Stellar batch-payout transaction using the Crossmint REST API.

Generates a transaction only. The caller handles the returned approvals.
The batch contract must already be deployed on the wallet's network.
"""

import requests


def create_batch_transaction(
    *,
    api_url: str,
    api_key: str,
    idempotency_key: str,
    wallet_address: str,
    signer_locator: str,
    batch_contract_id: str,
    token_contract_id: str,
    payments: list[dict[str, str]],
) -> dict[str, object]:
    """Reuse the persisted key and unchanged payload when retrying a batch."""
    if not idempotency_key.strip():
        raise ValueError("idempotency_key must be a non-empty persisted batch key")

    response = requests.post(
        f"{api_url.rstrip('/')}/wallets/{wallet_address}/transactions",
        headers={
            "X-API-KEY": api_key,
            "Content-Type": "application/json",
            "x-idempotency-key": idempotency_key,
        },
        json={
            "params": {
                "signer": signer_locator,
                "transaction": {
                    "type": "contract-call",
                    "contractId": batch_contract_id,
                    "method": "pay",
                    "args": {
                        "token": token_contract_id,
                        "from": wallet_address,
                        "payments": payments,
                    },
                },
            }
        },
        timeout=30,
    )
    response.raise_for_status()
    return response.json()
