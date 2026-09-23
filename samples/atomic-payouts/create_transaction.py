"""Copy into the USDM1 demo's server directory beside crossmint_client.py.

Generates a transaction only. The caller handles the returned approvals.
The batch contract must already be deployed on the wallet's network.
"""

import requests

from crossmint_client import CROSSMINT_API_KEY, CROSSMINT_API_URL


def create_batch_transaction(
    wallet_address: str,
    signer_locator: str,
    batch_contract_id: str,
    token_contract_id: str,
    payments: list[dict[str, str]],
) -> dict[str, object]:
    response = requests.post(
        f"{CROSSMINT_API_URL}/wallets/{wallet_address}/transactions",
        headers={
            "X-API-KEY": CROSSMINT_API_KEY,
            "Content-Type": "application/json",
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
