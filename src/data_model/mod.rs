use pyo3::prelude::*;

use self::account::*;

pub mod account;
pub mod asset;
pub mod block;
pub mod crypto;
pub mod domain;
pub mod role;

pub mod tx;

mod util;

pub trait PyMirror {
    type Mirror;

    fn mirror(self) -> PyResult<Self::Mirror>;
}

pub fn register_items(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    account::register_items(py, module)?;
    asset::register_items(py, module)?;
    domain::register_items(py, module)?;
    role::register_items(py, module)?;
    crypto::register_items(py, module)?;
    tx::register_items(py, module)?;
    block::register_items(py, module)?;
    Ok(())
}
