use iroha_data_model::domain::NewDomain;
use iroha_data_model::isi::{Mint, Register, Transfer, Unregister};
use iroha_data_model::prelude::*;
use iroha_primitives::numeric::Numeric;
use pyo3::{exceptions::PyValueError, prelude::*};

use std::str::FromStr;

use crate::data_model::asset::PyAssetType;
use rust_decimal::{prelude::FromPrimitive, Decimal};

#[derive(Debug, Clone)]
#[pyclass(name = "Instruction")]
pub struct PyInstruction(pub InstructionBox);

#[pymethods]
impl PyInstruction {
    #[staticmethod]
    /// Create an instruction for registering a new domain.
    fn register_domain(domain_id: &str) -> PyResult<PyInstruction> {
        let new_domain_object = NewDomain {
            id: DomainId::from_str(domain_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
            logo: None,
            metadata: Default::default(),
        };
        return Ok(PyInstruction(Register::domain(new_domain_object).into()));
    }

    #[staticmethod]
    /// Create an instruction for registering a new account.
    fn register_account(account_id: &str) -> PyResult<PyInstruction> {
        let new_account_object = Account::new(
            AccountId::from_str(account_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
        );
        return Ok(PyInstruction(Register::account(new_account_object).into()));
    }

    #[staticmethod]
    /// Create an instruction for registering a new account.
    fn register_asset_definition(
        asset_definition_id: &str,
        value_type: PyAssetType,
    ) -> PyResult<PyInstruction> {
        let new_definition_object = NewAssetDefinition {
            id: AssetDefinitionId::from_str(asset_definition_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
            type_: value_type.into(),
            mintable: Mintable::Infinitely,
            logo: None,
            metadata: Metadata::default(),
        };
        return Ok(PyInstruction(
            Register::asset_definition(new_definition_object).into(),
        ));
    }

    /* TODO(Sam): Replace this with explicit functions.
    #[staticmethod]
    /// Create an instruction to transfer ownership of an object
    /// Currently supported are: AssetDefinition and Domain
    fn transfer_ownership(
        py: Python<'_>,
        object: PyObject,
        from: PyAccountId,
        to: PyAccountId,
    ) -> PyResult<PyInstruction> {
        let (from, to) = (from.0, to.0);

        if let Ok(asset_definition_id) = object.extract::<PyAssetDefinitionId>(py) {
            return Ok(PyInstruction(
                Transfer::asset_definition(from, asset_definition_id.0, to).into(),
            ));
        }

        if let Ok(domain_id) = object.extract::<PyDomainId>(py) {
            return Ok(PyInstruction(
                Transfer::domain(from, domain_id.0, to).into(),
            ));
        }

        Err(PyValueError::new_err(
            "Only transferring ownership of accounts, asset definitions and domains is supported",
        ))
    }*/

    #[staticmethod]
    // Transfer asset value from one account to another
    fn transfer(py: Python<'_>, value: PyObject, from: &str, to: &str) -> PyResult<PyInstruction> {
        let from = AssetId::from_str(from).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let to = AccountId::from_str(to).map_err(|e| PyValueError::new_err(e.to_string()))?;

        let value = if let Ok(val) = value.extract::<u32>(py) {
            Numeric::new(val.into(), 0)
        } else if let Ok(val) = value.extract::<u128>(py) {
            Numeric::new(val, 0)
        } else if let Ok(val) = value.extract::<f64>(py) {
            let decimal = Decimal::from_f64(val).ok_or(PyValueError::new_err(
                "float could not be converted into decimal number",
            ))?;
            Numeric::new(decimal.mantissa() as u128, decimal.scale())
        } else {
            return Err(PyValueError::new_err(format!(
                "Unrecognised value for asset: {}",
                value
            )));
        };

        Ok(PyInstruction(
            Transfer::asset_numeric(from, value, to).into(),
        ))
    }

    #[staticmethod]
    // Mint value to an Asset
    fn mint_asset(py: Python<'_>, value: PyObject, asset_id: &str) -> PyResult<PyInstruction> {
        let value = if let Ok(val) = value.extract::<u32>(py) {
            Numeric::new(val.into(), 0)
        } else if let Ok(val) = value.extract::<u128>(py) {
            Numeric::new(val, 0)
        } else if let Ok(val) = value.extract::<f64>(py) {
            let decimal = Decimal::from_f64(val).ok_or(PyValueError::new_err(
                "float could not be converted into decimal number",
            ))?;
            Numeric::new(decimal.mantissa() as u128, decimal.scale())
        } else {
            return Err(PyValueError::new_err(format!(
                "Unrecognised value for asset: {}",
                value
            )));
        };
        // TODO(Sam): Add Store variant because it's needed for metadata.

        Ok(PyInstruction(
            Mint::asset_numeric(
                value,
                AssetId::from_str(asset_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
            )
            .into(),
        ))
    }

    #[staticmethod]
    /// Create an instruction for registering a new role.
    fn register_role(
        role_id: &str,
        grant_to_account_id: &str,
        permission_tokens: Vec<(&str, &str)>,
    ) -> PyResult<PyInstruction> {
        let mut role = Role::new(
            RoleId::from_str(role_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
            AccountId::from_str(grant_to_account_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        );
        for (definition_id, json_string) in permission_tokens {
            role = role.add_permission(Permission::new(
                iroha_schema::Ident::from_str(definition_id)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?,
                json_string,
            ));
        }
        return Ok(PyInstruction(Register::role(role).into()));
    }
    #[staticmethod]
    /// Create an instruction for unregistering a role.
    fn unregister_role(role_id: &str) -> PyResult<PyInstruction> {
        return Ok(PyInstruction(
            Unregister::role(
                RoleId::from_str(role_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
            )
            .into(),
        ));
    }
    #[staticmethod]
    /// Create an instruction for granting a role to an account.
    fn grant_role(role_id: &str, account_id: &str) -> PyResult<PyInstruction> {
        return Ok(PyInstruction(
            Grant::account_role(
                RoleId::from_str(role_id).map_err(|e| PyValueError::new_err(e.to_string()))?,
                AccountId::from_str(account_id)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?,
            )
            .into(),
        ));
    }
}

pub fn register_items(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyInstruction>()?;
    Ok(())
}
