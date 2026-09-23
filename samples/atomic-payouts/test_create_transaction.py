import unittest
from unittest.mock import Mock, patch

import requests

from create_transaction import create_batch_transaction


class CreateBatchTransactionTests(unittest.TestCase):
    def setUp(self):
        self.arguments = {
            "api_url": "https://staging.crossmint.com/api/2025-06-09/",
            "api_key": "test-only-placeholder",
            "idempotency_key": "persisted-batch-a",
            "wallet_address": "<SOURCE_WALLET_ADDRESS>",
            "signer_locator": "external-wallet:<REGISTERED_SIGNER_PUBLIC_KEY>",
            "batch_contract_id": "<BATCH_CONTRACT_ID>",
            "token_contract_id": "<TOKEN_CONTRACT_ID>",
            "payments": [{"to": "<RECIPIENT_ADDRESS>", "amount": "10000000"}],
        }

    @patch("create_transaction.requests.post")
    def test_timeout_retry_preserves_key_and_payload(self, post):
        response = Mock()
        transaction = {
            "id": "original-transaction",
            "status": "awaiting-approval",
            "approvals": {"pending": [{"signer": self.arguments["signer_locator"]}]},
        }
        response.json.return_value = transaction
        post.side_effect = [requests.Timeout("response lost"), response]

        with self.assertRaises(requests.Timeout):
            create_batch_transaction(**self.arguments)
        self.assertEqual(create_batch_transaction(**self.arguments), transaction)

        first, retry = post.call_args_list
        self.assertEqual(first, retry)
        self.assertEqual(first.kwargs["headers"]["x-idempotency-key"], "persisted-batch-a")
        self.assertEqual(
            first.args[0],
            "https://staging.crossmint.com/api/2025-06-09/wallets/<SOURCE_WALLET_ADDRESS>/transactions",
        )
        call = first.kwargs["json"]["params"]["transaction"]
        self.assertEqual(call["args"]["payments"], self.arguments["payments"])
        response.raise_for_status.assert_called_once()

    @patch("create_transaction.requests.post")
    def test_new_batch_uses_a_new_caller_key(self, post):
        create_batch_transaction(**self.arguments)
        create_batch_transaction(**{**self.arguments, "idempotency_key": "persisted-batch-b"})
        keys = [call.kwargs["headers"]["x-idempotency-key"] for call in post.call_args_list]
        self.assertEqual(keys, ["persisted-batch-a", "persisted-batch-b"])

    @patch("create_transaction.requests.post")
    def test_missing_or_empty_key_never_sends_a_request(self, post):
        arguments = {key: value for key, value in self.arguments.items() if key != "idempotency_key"}
        with self.assertRaises(TypeError):
            create_batch_transaction(**arguments)
        for key in ["", " ", "\t"]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                create_batch_transaction(**{**arguments, "idempotency_key": key})
        post.assert_not_called()

    @patch("create_transaction.requests.post")
    def test_http_rejection_is_not_returned_as_a_transaction(self, post):
        post.return_value.raise_for_status.side_effect = requests.HTTPError("rejected")
        with self.assertRaises(requests.HTTPError):
            create_batch_transaction(**self.arguments)
        post.return_value.json.assert_not_called()


if __name__ == "__main__":
    unittest.main()
