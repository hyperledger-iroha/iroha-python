import allure
import iroha2
import pytest

from tests import client

@pytest.fixture(scope="function", autouse=True)
def story_account_registers_domain():
    allure.dynamic.story("Account registers a domain")
    allure.dynamic.label("permission", "no_permission_required")

@allure.id("2382")
@allure.label("sdk_test_id", "register_domain")
def test_register_domain(
        GIVEN_new_domain_id):
    with allure.step(f'WHEN client registers the domain name "{GIVEN_new_domain_id}"'):
        (client.submit_executable_only_success(
            [iroha2.Instruction
             .register_domain(GIVEN_new_domain_id)]))
    with allure.step(f'THEN Iroha should have the domain name "{GIVEN_new_domain_id}"'):
        assert GIVEN_new_domain_id in client.query_all_domains()
