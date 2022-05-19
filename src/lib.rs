//! iroha2-python sys library with all classes (wrapped rust structures) with methods

// Allow panic because of bad and unsafe pyo3
#![allow(
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    clippy::multiple_inherent_impl
)]

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use color_eyre::eyre;
use iroha_client::{client, config::Configuration};
use iroha_crypto::{Hash, KeyGenConfiguration, Signature};
use iroha_crypto::{PrivateKey, PublicKey};
use iroha_data_model::prelude::*;
use iroha_version::prelude::*;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::iter::IterNextOutput;
use pyo3::prelude::*;
use pyo3::PyIterProtocol;

use crate::python::*;

mod python;
mod types;

#[pymethods]
impl KeyPair {
    /// Generates new key
    /// # Errors
    #[new]
    pub fn generate() -> PyResult<Self> {
        iroha_crypto::KeyPair::generate()
            .map_err(to_py_err)
            .map(Into::into)
    }

    /// Create keypair with some seed
    /// # Errors
    #[staticmethod]
    pub fn with_seed(seed: Vec<u8>) -> PyResult<Self> {
        let cfg = KeyGenConfiguration::default().use_seed(seed);
        iroha_crypto::KeyPair::generate_with_configuration(cfg)
            .map_err(to_py_err)
            .map(Into::into)
    }

    /// Gets public key
    #[getter]
    pub fn public(&self) -> ToPy<PublicKey> {
        ToPy(self.public_key().clone())
    }

    /// Gets private key
    #[getter]
    pub fn private(&self) -> ToPy<PrivateKey> {
        ToPy(self.private_key().clone())
    }
}

/// Hash bytes
#[pyfunction]
pub fn hash(bytes: Vec<u8>) -> ToPy<Hash> {
    ToPy(Hash::new(&bytes))
}

/// TODO: signing
#[pymethods]
impl Client {
    /// Creates new client
    #[new]
    pub fn new(cfg: ToPy<Configuration>) -> Self {
        // TODO:
        client::Client::new(&cfg).unwrap().into()
    }

    /// Queries peer
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn request(&mut self, query: ToPy<QueryBox>) -> PyResult<ToPy<Value>> {
        self.deref_mut()
            .request(query.into_inner())
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Get transaction body
    /// # Errors
    pub fn tx_body(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<Vec<u8>> {
        let isi = isi.into_iter().map(ToPy::into_inner).into();
        self.build_transaction(isi, metadata.into_inner())
            .map(VersionedTransaction::from)
            .map_err(to_py_err)
            .map(|tx| tx.encode_versioned())
    }

    // TODO:
    /// Get transaction body
    /// # Errors
    //  pub fn query_body(&mut self, request: ToPy<QueryBox>) -> PyResult<Vec<u8>> {
    //      let request = QueryRequest::new(request.into_inner(), self.cl.account_id.clone());
    //      let signed = self.cl.sign_query(request);
    //      signed
    //          .map(VersionedSignedQueryRequest::from)
    //          .map_err(to_py_err)
    //          .map(|req| req.encode_versioned())
    //  }

    /// Sends transaction to peer
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn submit_all_with_metadata(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<ToPy<Hash>> {
        let isi = isi.into_iter().map(ToPy::into_inner);
        self.deref_mut()
            .submit_all_with_metadata(isi, metadata.into_inner())
            .map(|h| *h)
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Sends transaction to peer and waits till its finalization
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn submit_all_blocking_with_metadata(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<ToPy<Hash>> {
        let isi = isi.into_iter().map(ToPy::into_inner);
        self.deref_mut()
            .submit_all_blocking_with_metadata(isi, metadata.into_inner())
            .map(|h| *h)
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Listen on web socket events
    pub fn listen_for_events(&mut self, event_filter: ToPy<FilterBox>) -> PyResult<EventIterator> {
        self.deref_mut()
            .listen_for_events(event_filter.into_inner())
            .map_err(to_py_err)
            .map(|iter| {
                let boxed = Box::new(iter);
                EventIterator::new(boxed)
            })
    }

    // TODO:
    // /// Account field on client
    // #[getter]
    // pub fn get_account(&self) -> ToPy<AccountId> {
    //     ToPy(self.account_id.clone())
    // }

    // TODO:
    // /// Account field on client
    // #[setter]
    // pub fn set_account(&mut self, account: ToPy<AccountId>) {
    //   self.account_id = account.into_inner();
    // }

    // TODO:
    // /// Headers field on client
    // #[getter]
    // pub fn get_headers(&self) -> HashMap<String, String> {
    //     self.headers.clone()
    // }

    // TODO:
    // /// Account field on client
    // #[setter]
    // pub fn set_headers(&mut self, headers: HashMap<String, String>) {
    //     self.headers = headers;
    // }
}

// TODO:
// `EventIterator` was made private in iroha for some reason
#[pyclass]
pub struct EventIterator {
    inner: Box<dyn Iterator<Item = eyre::Result<Event>> + Send>,
}

impl EventIterator {
    fn new(inner: Box<dyn Iterator<Item = eyre::Result<Event>> + Send>) -> Self {
        Self { inner }
    }
}

#[pyproto]
impl PyIterProtocol for EventIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    // TODO
    fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<ToPy<Event>, &'static str> {
        #[allow(clippy::unwrap_used)]
        slf.inner
            .next()
            .map(Result::unwrap)
            .map(ToPy)
            .map_or(IterNextOutput::Return("Ended"), IterNextOutput::Yield)
    }
}

#[rustfmt::skip]
wrap_class!(
    KeyPair        { keys: iroha_crypto::KeyPair   }: Debug + Clone,
    Client         { cl:   client::Client          }: Debug + Clone,
);

/// A Python module implemented in Rust.
#[pymodule]
pub fn iroha2(_: Python, m: &PyModule) -> PyResult<()> {
    register_wrapped_classes(m)?;
    m.add_class::<types::Dict>()?;
    m.add_class::<types::List>()?;
    Ok(())
}
