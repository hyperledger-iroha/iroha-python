import re

import allure
import iroha
import time

import pytest

from tests import client

@pytest.fixture(scope="function", autouse=True)
def story_account_register_asset():
    allure.dynamic.story("Account registers an asset")
    allure.dynamic.label("permission", "no_permission_required")

@allure.id("2383")
def test_register_asset_definition(
        GIVEN_new_asset_definition_id):
    with allure.step(
            f'WHEN client registers a new asset definition id "{GIVEN_new_asset_definition_id}"'):
        (client.submit_executable_only_success(
            [iroha.Instruction
             .register_asset_definition(
                GIVEN_new_asset_definition_id,
                iroha.AssetValueType.numeric_fractional(0))]))
    with allure.step(
            f'THEN Iroha should have the "{GIVEN_new_asset_definition_id}" account'):
        assert GIVEN_new_asset_definition_id in client.query_all_asset_definitions()

@allure.id("2379")
def test_mint_asset(
    GIVEN_registered_asset_definition,
    GIVEN_registered_account):
    asset = (lambda s: re.sub(r'(\b\w+\b)(?=.*\1)', '', s))(GIVEN_registered_asset_definition + '#' + GIVEN_registered_account)
    with allure.step(
            f'WHEN client mints an asset "{asset}"'):
        (client.submit_executable_only_success(
            [iroha.Instruction
             .mint_asset(
                5,
                asset)]))
    with allure.step(
            f'THEN Iroha should have the new asset "{asset}"'):
        assert asset in client.query_all_assets()