import allure  # type: ignore
import pytest

@pytest.fixture(scope="function", autouse=True)
def asset_test_setup():
    allure.dynamic.feature("Accounts")
    allure.dynamic.label("permission", "no_permission_required")
