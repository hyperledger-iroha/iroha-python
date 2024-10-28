use rust_decimal::{prelude::FromPrimitive, Decimal};

use iroha_data_model::asset::{
    Asset, AssetDefinition, AssetDefinitionId, AssetId, AssetType, AssetValue, Mintable,
    NewAssetDefinition,
};
use iroha_primitives::numeric::{Numeric, NumericSpec};

use pyo3::{
    exceptions::{PyNotImplementedError, PyValueError},
    prelude::*,
    types::PyDict,
};

use super::account::PyAccountId;
use crate::{data_model::PyMirror, mirror_fieldless_enum, mirror_struct};

mirror_struct! {
    /// ID of asset definition, e.g. asset#domain
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    AssetDefinitionId
}

#[pymethods]
impl PyAssetDefinitionId {
    #[new]
    fn new(name: &str, domain: &str) -> PyResult<Self> {
        let name = name
            .parse()
            .map_err(|e| PyValueError::new_err(format!("Invalid AssedDefinitionId name: {e}")))?;
        let domain = domain
            .parse()
            .map_err(|e| PyValueError::new_err(format!("Invalid Domain name: {e}")))?;
        Ok(Self(AssetDefinitionId::new(name, domain)))
    }

    #[getter]
    fn get_name(&self) -> &str {
        self.0.name().as_ref()
    }

    #[setter]
    fn set_name(&mut self, name: &str) -> PyResult<()> {
        self.0.name = name
            .parse()
            .map_err(|e| PyValueError::new_err(format!("Invalid AssedDefinitionId name: {e}")))?;
        Ok(())
    }

    #[getter]
    fn get_domain(&self) -> &str {
        self.0.domain().name().as_ref()
    }

    #[setter]
    fn set_domain(&mut self, name: &str) -> PyResult<()> {
        self.0.domain.name = name
            .parse()
            .map_err(|e| PyValueError::new_err(format!("Invalid Domain name: {e}")))?;
        Ok(())
    }
}

mirror_struct! {
    /// Asset definition, repesenting existence of
    /// a particular type of asset on the blockchain.
    ///
    /// By analogy to fiat currency, AssetDefinition would
    /// be e.g. USD as a fiat currency, while an Asset is a
    /// USD balance of a particular person
    ///
    /// Please note that for registering a new asset definition
    /// you should use NewAssetDefinition rather than this class
    AssetDefinition
}

#[pymethods]
impl PyAssetDefinition {
    #[getter]
    fn get_id(&self) -> PyAssetDefinitionId {
        self.0.id.clone().into()
    }

    #[getter]
    fn get_type(&self) -> PyAssetType {
        self.0.type_.into()
    }

    #[getter]
    fn get_mintable(&self) -> PyMintable {
        self.0.mintable.into()
    }

    #[getter]
    fn get_logo(&self) -> Option<&str> {
        self.0.logo.as_ref().map(|logo| logo.as_ref())
    }

    #[getter]
    fn get_owned_by(&self) -> PyAccountId {
        self.0.owned_by.clone().into()
    }
}

mirror_struct! {
    /// Class for registering a new AssetDefinition
    NewAssetDefinition
}

#[pymethods]
impl PyNewAssetDefinition {
    #[new]
    fn new(
        py: Python<'_>,
        id: PyObject,
        value_type: PyAssetType,
        mintable: Option<PyMintable>,
        logo: Option<String>,
        metadata: Option<Py<PyDict>>,
    ) -> PyResult<Self> {
        let id = if let Ok(defn_id) = id.extract::<PyAssetDefinitionId>(py) {
            defn_id.into()
        } else if let Ok(str_id) = id.extract::<&str>(py) {
            str_id
                .parse()
                .map_err(|e| PyValueError::new_err(format!("Invalid AssetDefinition id: {e}")))?
        } else {
            return Err(PyValueError::new_err(
                "Invalid AssetDefinition id, expected AssetDefinitionId or a string",
            ));
        };
        let mut new_definition = AssetDefinition::new(id, value_type.into());
        if let Some(mintable) = mintable {
            new_definition.mintable = mintable.into();
        }
        if let Some(logo) = logo {
            let logo = logo
                .parse()
                .map_err(|e| PyValueError::new_err(format!("Invalid IPFS path: {e}")))?;
            new_definition.logo = Some(logo);
        }
        if let Some(_metadata) = metadata {
            todo!()
        }
        Ok(Self(new_definition))
    }

    #[getter]
    fn get_id(&self) -> PyAssetDefinitionId {
        self.0.id.clone().into()
    }

    #[setter]
    fn set_id(&mut self, py: Python<'_>, id: PyObject) -> PyResult<()> {
        if let Ok(s) = id.extract::<&str>(py) {
            self.0.id = s
                .parse()
                .map_err(|e| PyValueError::new_err(format!("Invalid AssetDefinition: {e}")))?;
        } else {
            self.0.id = id.extract::<PyAssetDefinitionId>(py)?.into();
        }
        Ok(())
    }

    #[getter]
    fn get_type(&self) -> PyAssetType {
        self.0.type_.into()
    }

    #[setter]
    fn set_type(&mut self, new: PyAssetType) -> () {
        self.0.type_ = new.into();
    }

    #[getter]
    fn get_mintable(&self) -> PyMintable {
        self.0.mintable.into()
    }

    #[setter]
    fn set_mintable(&mut self, new: PyAssetType) -> () {
        self.0.type_ = new.into();
    }

    #[getter]
    fn get_logo(&self) -> Option<&str> {
        self.0.logo.as_ref().map(|logo| logo.as_ref())
    }

    #[setter]
    fn set_logo(&mut self, new: Option<String>) -> PyResult<()> {
        if let Some(path) = new {
            self.0.logo = Some(
                path.parse()
                    .map_err(|e| PyValueError::new_err(format!("Malformed IPFS path: {e}")))?,
            )
        }
        Ok(())
    }
}

mirror_struct! {
    /// AssetId, comprised of AssetDefinitionId
    /// and particular Asset it belongs to
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    AssetId
}

#[pymethods]
impl PyAssetId {
    #[new]
    fn new(definition_id: PyAssetDefinitionId, account_id: PyAccountId) -> PyResult<Self> {
        Ok(Self(AssetId::new(definition_id.into(), account_id.into())))
    }

    #[getter]
    fn get_definition_id(&self) -> PyAssetDefinitionId {
        self.0.definition.clone().into()
    }

    #[setter]
    fn set_definition_id(&mut self, definition_id: PyAssetDefinitionId) {
        self.0.definition = definition_id.into()
    }

    #[getter]
    fn get_account_id(&self) -> PyAccountId {
        self.0.account.clone().into()
    }

    #[setter]
    fn set_account_id(&mut self, account_id: PyAccountId) {
        self.0.account = account_id.into()
    }
}

mirror_struct! {
    /// Asset balance belonging to an account
    Asset
}

#[pymethods]
impl PyAsset {
    #[new]
    fn new(py: Python<'_>, id: PyAssetId, value: PyObject) -> PyResult<Self> {
        let value = if let Ok(val) = value.extract::<u32>(py) {
            AssetValue::Numeric(Numeric::new(val.into(), 0))
        } else if let Ok(val) = value.extract::<u128>(py) {
            AssetValue::Numeric(Numeric::new(val, 0))
        } else if let Ok(val) = value.extract::<f64>(py) {
            let decimal = Decimal::from_f64(val).ok_or(PyValueError::new_err(
                "float could not be converted into decimal number",
            ))?;
            AssetValue::Numeric(Numeric::new(decimal.mantissa() as u128, decimal.scale()))
        } else {
            return Err(PyValueError::new_err(format!(
                "Unrecognised value for asset: {}",
                value
            )));
        };

        Ok(Self(Asset::new(id.0, value)))
    }

    #[getter]
    fn get_id(&self) -> PyAssetId {
        self.0.id.clone().into()
    }

    #[setter]
    fn set_id(&mut self, id: PyAssetId) {
        self.0.id = id.into()
    }

    #[getter]
    fn get_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.0.value {
            AssetValue::Numeric(n) => {
                let quantity =
                    Decimal::from_i128_with_scale(n.mantissa() as i128, n.scale()).into_py(py);
                Ok(quantity.into())
            }
            AssetValue::Store(_v) => {
                //let dict = MetadataWrapper(v.clone()).into_py(py)?;
                //Ok(dict.into())
                unimplemented!();
            }
        }
    }

    #[setter]
    fn set_value(&mut self, py: Python<'_>, value: PyObject) -> PyResult<()> {
        let value = if let Ok(val) = value.extract::<u32>(py) {
            AssetValue::Numeric(Numeric::new(val.into(), 0))
        } else if let Ok(val) = value.extract::<u128>(py) {
            AssetValue::Numeric(Numeric::new(val, 0))
        } else if let Ok(val) = value.extract::<f64>(py) {
            let decimal = Decimal::from_f64(val).ok_or(PyValueError::new_err(
                "float could not be converted into decimal number",
            ))?;
            AssetValue::Numeric(Numeric::new(decimal.mantissa() as u128, decimal.scale()))
        } else {
            return Err(PyNotImplementedError::new_err(
                "Metadata Values are currently read-only",
            ));
        };
        self.0.value = value;
        Ok(())
    }
}

#[pyclass(name = "AssetType")]
#[derive(Clone, derive_more::From, derive_more::Into, derive_more::Deref)]
pub struct PyAssetType(pub AssetType);

#[pymethods]
impl PyAssetType {
    #[staticmethod]
    fn numeric_unconstrained() -> PyResult<Self> {
        Ok(Self(AssetType::Numeric(NumericSpec::unconstrained())))
    }
    #[staticmethod]
    fn numeric_fractional(scale: u32) -> PyResult<Self> {
        Ok(Self(AssetType::Numeric(NumericSpec::fractional(scale))))
    }
    #[staticmethod]
    fn store() -> PyResult<Self> {
        Ok(Self(AssetType::Store))
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl PyMirror for AssetType {
    type Mirror = PyAssetType;

    fn mirror(self) -> PyResult<Self::Mirror> {
        Ok(PyAssetType(self))
    }
}

mirror_fieldless_enum! {
    /// Various modes controlling asset mintability
    Mintable;
    Infinitely, Once, Not
}

pub fn register_items(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyAssetDefinitionId>()?;
    module.add_class::<PyAssetDefinition>()?;
    module.add_class::<PyNewAssetDefinition>()?;
    module.add_class::<PyAssetId>()?;
    module.add_class::<PyAsset>()?;
    module.add_class::<PyAssetType>()?;
    module.add_class::<PyMintable>()?;
    Ok(())
}
