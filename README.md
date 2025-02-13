# Iroha Python

Python library for **Hyperledger Iroha 2**.

This README provides essential information on how to install, build, and test the library, as well as useful references for deeper exploration.

## Table of Contents
1. [Description](#description)  
2. [Library Version](#library-version)  
3. [Environment Requirements](#environment-requirements)  
4. [Installation and Build](#installation-and-build)  
5. [Verification of Installation](#verification-of-installation)  
6. [Running Tests](#running-tests)  
7. [Additional Resources](#additional-resources)

## Description

This library offers a Python interface for **Hyperledger Iroha 2**, providing classes and methods needed to integrate Iroha functionality into your Python projects. It covers core entities such as `Account`, `Asset`, `Domain`, `Transaction`, and many others.

## Library Version

- If you are using the **latest Iroha release** (iroha2 MVP), please use the `main` branch.  
- If you are using **rc20**, switch to the `stable` branch.

## Environment Requirements

- **Rust** toolchain pinned to `nightly-2024-09-09` (necessary for building the library).  
- **Python 3.12** (based on the provided wheel file).  
- **Poetry** (for running tests) and any other dependencies required by `maturin` for building.  
- A functional local instance of **Hyperledger Iroha 2** for integration and testing purposes.

## Installation and Build

1. **Set the Rust toolchain** to the required nightly version:
   ```sh
   rustup override set nightly-2024-09-09
   ```

2. **Build** the library with `maturin`:
   ```sh
   maturin build
   ```
   This command generates a `.whl` file in the `target/wheels/` directory.

3. **Install** the generated package:
   ```sh
   pip install --break-system-packages target/wheels/iroha-0.1.0-cp312-cp312-manylinux_2_34_x86_64.whl
   ```
   The exact filename may differ depending on your system and Python version.

## Verification of Installation

To verify successful installation:
```sh
python -c "import iroha; print(dir(iroha))"
```
If the library is correctly installed, you should see output similar to:
```
['Account', 'AccountId', 'Asset', 'AssetDefinition', 'AssetDefinitionId', 'AssetId', 'AssetType', 'BlockHeader', 'Client', 'DomainId', 'Instruction', 'KeyPair', 'Mintable', 'NewAccount', 'NewAssetDefinition', 'PrivateKey', 'PublicKey', 'Role', 'SignedTransaction', 'TransactionQueryOutput', '__all__', '__builtins__', '__cached__', '__doc__', '__file__', '__loader__', '__name__', '__package__', '__path__', '__spec__', 'hash', 'iroha']
```
If you encounter an error like:
```
ModuleNotFoundError: No module named 'iroha'
```
it means the installation did not succeed correctly, and Python cannot locate the package. Double-check that you installed the wheel in the same Python environment where you’re running the command.

## Running Tests

1. In the main Iroha repository, set up the local test environment:
   ```sh
   scripts/test_env.py setup
   ```
   This prepares a local Iroha test network.

2. In this repository (the Iroha Python one), run:
   ```sh
   poetry run python -m pytest tests/
   ```
   This command will execute the test suite, verifying the library’s functionality.

## Additional Resources

- [Hyperledger Iroha 2 Documentation](https://github.com/hyperledger/iroha)