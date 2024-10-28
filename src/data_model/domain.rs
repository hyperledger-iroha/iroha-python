use iroha_data_model::domain::prelude::*;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::data_model::account::*;
use crate::mirror_struct;

mirror_struct! {
    /// Domain id
    DomainId
}

#[pymethods]
impl PyDomainId {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        let name = name
            .parse()
            .map_err(|e| PyValueError::new_err(format!("Invalid Domain name: {e}")))?;
        Ok(Self(DomainId::new(name)))
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
}

mirror_struct! {
    /// Domain id
    Domain
}

#[pymethods]
impl PyDomain {
    #[getter]
    fn get_id(&self) -> PyDomainId {
        PyDomainId(self.0.id.clone())
    }

    #[getter]
    fn get_logo(&self) -> Option<&str> {
        self.0.logo.as_ref().map(|logo| logo.as_ref())
    }

    #[getter]
    fn get_owned_by(&self) -> PyAccountId {
        self.0.owned_by.clone().into()
    }

    #[getter]
    fn get_metadata(&self, _py: Python<'_>) -> PyResult<Py<PyDict>> {
        //MetadataWrapper(self.0.metadata.clone()).into_py(py)
        unimplemented!();
    }
}

pub fn register_items(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyDomainId>()?;
    Ok(())
}
