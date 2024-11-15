import re

import allure
import iroha2

import pytest

from tests import client

@pytest.fixture(scope="function", autouse=True)
def story_account_register_asset():
    allure.dynamic.story("Account mints an asset")

@allure.id("2379")
@allure.label("sdk_test_id", "mint_asset")
def test_mint_asset(
    GIVEN_registered_asset_definition,
    GIVEN_registered_account):
    asset = (lambda s: re.sub(r'(\b\w+\b)(?=.*\1)', '', s))(GIVEN_registered_asset_definition + '#' + GIVEN_registered_account)
    with allure.step(
            f'WHEN client mints an asset "{asset}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .mint_asset(
                5,
                asset)]))
    with allure.step(
            f'THEN Iroha should have the new asset "{asset}"'):
        assert asset in client.query_all_assets()
