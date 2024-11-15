use pyo3::prelude::*;

mod client;
mod data_model;
mod isi;

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "iroha2")]
fn iroha2_python(py: Python, m: &PyModule) -> PyResult<()> {
    client::register_items(py, m)?;
    data_model::register_items(py, m)?;
    isi::register_items(py, m)?;

    Ok(())
}
