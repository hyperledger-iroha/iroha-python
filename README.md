# Iroha python

Python library for Hyperledger Iroha 2.

## Version

If you are using the latest iroha release, iroha2 MVP, then you should use the `main` branch. If you are using rc20 then you should use the 'stable' branch.

## Install

To build, use the nightly-2024-09-09 version of the rust toolchain. Set it as the default before executing the build steps. TODO, complete list of dependencies.

```sh
maturin build
pip install --break-system-packages target/wheels/iroha-0.1.0-cp312-cp312-manylinux_2_34_x86_64.whl
```

The exact path to the .whl file may vary. After an installation, do a small test to check the installation was successful. Normally, this test will display the library's contents:

```
python -c "import iroha2; print(dir(iroha2))"
['Account', 'AccountId', 'Asset', 'AssetDefinition', 'AssetDefinitionId', 'AssetId', 'AssetType', 'BlockHeader', 'Client', 'CommittedTransaction', 'DomainId', 'Instruction', 'KeyPair', 'Mintable', 'NewAssetDefinition', 'PrivateKey', 'PublicKey', 'Role', 'SignedTransaction', '__all__', '__builtins__', '__cached__', '__doc__', '__file__', '__loader__', '__name__', '__package__', '__path__', '__spec__', 'hash', 'iroha2']
```

If there was no error, you should be able to use iroha-python library.
You may also see an error that ends like this:

```
ModuleNotFoundError: No module named 'iroha2'
```

This means that the pip install did not work properly.

## Running the tests

Running the tests requires you to have a running local test network of iroha. In the main iroha repository you must run 'scripts/test_env.py setup'. Then you can run the following command to run the python library test suite.

```sh
poetry run python -m pytest tests/
```