import allure
import iroha2

import pytest

from tests import client

@pytest.fixture(scope="function", autouse=True)
def story_account_register_asset():
    allure.dynamic.story("Account registers an asset definition")
    allure.dynamic.label("permission", "no_permission_required")

@allure.id("2383")
@allure.label("sdk_test_id", "register_asset_definition")
def test_register_asset_definition(
        GIVEN_new_asset_definition_id):
    with allure.step(
            f'WHEN client registers a new asset definition id "{GIVEN_new_asset_definition_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .register_asset_definition(
                GIVEN_new_asset_definition_id,
                iroha2.AssetType.numeric_fractional(0))]))
    with allure.step(
            f'THEN Iroha should have the "{GIVEN_new_asset_definition_id}" account'):
        assert GIVEN_new_asset_definition_id in client.query_all_asset_definitions()
