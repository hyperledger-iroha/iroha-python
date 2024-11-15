import re
import time

import allure
import iroha2
import pytest

from tests import client, fake
from tests.helpers import generate_public_key

@pytest.fixture(scope="function", autouse=True)
def before_each():
    """Fixture to set up and reset the client_cli state."""
    allure.dynamic.label("sdk", "Python 3")
    allure.dynamic.label("owner", "astrokov")

# Fixtures for Generating New Identifiers
@pytest.fixture()
def GIVEN_new_domain_id():
    """Fixture to provide a new fake domain id."""
    name = fake.country().split()[0] + str(len(client.query_all_domains()))
    with allure.step(f'GIVEN a "{name}" name'):
        return name

@pytest.fixture()
def GIVEN_new_account_id(GIVEN_registered_domain):
    """Fixture to provide a new fake account id."""
    name = str(generate_public_key(seed=format(len(client.query_all_accounts()), '08x'))) + '@' + GIVEN_registered_domain
    with allure.step(f'GIVEN a "{name}" name'):
        return name

@pytest.fixture()
def GIVEN_new_asset_id():
    """Fixture to provide a new asset id."""
    asset_name = fake.word()[:3].upper() + str(len(client.query_all_asset_definitions()))
    with allure.step(f'GIVEN a "{asset_name}" asset'):
        return asset_name

@pytest.fixture()
def GIVEN_new_asset_definition_id(GIVEN_new_asset_id, GIVEN_registered_domain):
    """Fixture to provide a new asset definition id."""
    asset_name = GIVEN_new_asset_id + "#" + GIVEN_registered_domain
    with allure.step(f'GIVEN a "{asset_name}" asset'):
        return asset_name

# Fixtures for Registering Entities
@pytest.fixture()
def GIVEN_registered_asset_definition(GIVEN_new_asset_definition_id):
    """Fixture to provide a registered asset definition."""
    with allure.step(
            f'GIVEN registered asset definition "{GIVEN_new_asset_definition_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
            .register_asset_definition(
                GIVEN_new_asset_definition_id,
                iroha2.AssetType.numeric_fractional(0))]))
        return GIVEN_new_asset_definition_id

@pytest.fixture()
def GIVEN_registered_domain(GIVEN_new_domain_id):
    """Fixture to provide a registered domain in Iroha"""
    with allure.step(f'GIVEN registered domain name "{GIVEN_new_domain_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .register_domain(GIVEN_new_domain_id)]))
        return GIVEN_new_domain_id

@pytest.fixture()
def GIVEN_registered_account(GIVEN_new_account_id):
    """Fixture to provide a registered account"""
    with allure.step(
            f'GIVEN client registered the account "{GIVEN_new_account_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .register_account(GIVEN_new_account_id)]))
        return GIVEN_new_account_id

@pytest.fixture()
def GIVEN_registered_domain_with_registered_accounts(
        GIVEN_registered_domain,
        GIVEN_new_account_id):
    """Fixture to provide a domain with accounts"""
    with allure.step(
            f'GIVEN client registered the account "{GIVEN_new_account_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .register_account(GIVEN_new_account_id)]))
        return GIVEN_registered_domain

@pytest.fixture()
def GIVEN_registered_account_with_minted_assets(
        GIVEN_registered_asset_definition,
        GIVEN_registered_account
):
    """Fixture to provide an account with minted assets"""
    asset = (lambda s: re.sub(r'(\b\w+\b)(?=.*\1)', '', s))(GIVEN_registered_asset_definition + '#' + GIVEN_registered_account)
    with allure.step(
            f'GIVEN client minted an asset "{asset}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
            .mint_asset(
                5,
                asset)]))
    return GIVEN_registered_account

@pytest.fixture()
def GIVEN_minted_asset(
    GIVEN_registered_asset_definition,
    GIVEN_registered_account):
    asset = (lambda s: re.sub(r'(\b\w+\b)(?=.*\1)', '', s))(GIVEN_registered_asset_definition + '#' + GIVEN_registered_account)
    with allure.step(
            f'GIVEN client mints an asset "{asset}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .mint_asset(
                5,
                asset)]))
    return asset


@pytest.fixture()
def GIVEN_hash_of_registered_transaction(GIVEN_new_domain_id):
    """Fixture to provide a registered transaction in Iroha"""
    with allure.step(f'GIVEN registered domain name "{GIVEN_new_domain_id}"'):
        tx_hash_string = client.submit_executable_only_success(
            [iroha2.Instruction
             .register_domain(GIVEN_new_domain_id)])
        return tx_hash_string
