import allure
import iroha2

from tests import client,account_id
from tests.helpers import generate_public_key


@allure.label("sdk_test_id", "query_all_transactions")
def test_query_all_transactions(GIVEN_hash_of_registered_transaction):
    with allure.step('WHEN client queries all transactions'):
        all_txs = client.query_all_transactions()
    with allure.step('THEN there should be some transactions present'):
        assert len(all_txs) > 0, "No transactions found in the system"


@allure.label("sdk_test_id", "query_all_transactions_by_account")
def test_query_all_transactions_by_account(
        GIVEN_hash_of_registered_transaction):
    with allure.step('WHEN client queries all transactions by their own account'):
        all_txs = client.query_all_transactions_by_account(account_id)
    with allure.step('THEN there should be some transactions present'):
        assert len(all_txs) > 0, "No transactions found in the system"

@allure.label("sdk_test_id", "test_query_transaction_by_hash")
def test_query_transaction_by_hash(
        GIVEN_hash_of_registered_transaction):
    with allure.step(
            f'WHEN client queries for transaction by hash {GIVEN_hash_of_registered_transaction}"'):
        tx = client.query_transaction_by_hash(bytes.fromhex(GIVEN_hash_of_registered_transaction))
    with allure.step('THEN there should be a transaction present'):
        assert tx is not None, "No transactions found in the system"
