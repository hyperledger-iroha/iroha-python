use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

use iroha::client::Client as IrohaClient;
use iroha::config::Config as IrohaClientConfig;
use iroha::config::{BasicAuth, WebLogin};

use std::num::NonZeroU64;
use std::str::FromStr;

use crate::data_model::block::*;
use crate::data_model::crypto::*;
use crate::data_model::role::*;
use crate::data_model::tx::*;
use crate::isi::PyInstruction;
use iroha_crypto::{Hash, HashOf};
use iroha_data_model::account::AccountId;
use iroha_data_model::prelude::*;
use iroha_data_model::ChainId;

use iroha_data_model::events::pipeline::{BlockEventFilter, TransactionEventFilter};

use iroha_data_model::query;

#[allow(unsafe_code)]
const DEFAULT_TRANSACTION_TIME_TO_LIVE_MS: NonZeroU64 =
    unsafe { NonZeroU64::new_unchecked(100_000) };
const DEFAULT_TRANSACTION_STATUS_TIMEOUT_MS: u64 = 15_000;
const DEFAULT_ADD_TRANSACTION_NONCE: bool = false;

#[pyclass]
pub struct Client {
    client: IrohaClient,
}

#[pymethods]
impl Client {
    #[staticmethod]
    fn create(
        key_pair: &PyKeyPair,
        account_id: &str,
        web_login: &str,
        password: &str,
        api_url: &str,
        chain_id: &str,
    ) -> PyResult<Self> {
        let config = IrohaClientConfig {
            chain: ChainId::from(chain_id),
            account: AccountId::from_str(account_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
            key_pair: key_pair.clone().into(),
            basic_auth: Some(BasicAuth {
                web_login: WebLogin::from_str(web_login)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?,
                password: iroha_primitives::small::SmallStr::from_str(password),
            }),
            torii_api_url: url::Url::parse(api_url)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
            transaction_ttl: std::time::Duration::from_millis(
                DEFAULT_TRANSACTION_TIME_TO_LIVE_MS.into(),
            ),
            transaction_status_timeout: std::time::Duration::from_millis(
                DEFAULT_TRANSACTION_STATUS_TIMEOUT_MS,
            ),
            // deprecated, does nothing.
            transaction_add_nonce: DEFAULT_ADD_TRANSACTION_NONCE,
        };
        Ok(Self {
            client: IrohaClient::new(config),
        })
    }

    fn submit_executable(&self, py: Python<'_>, isi: PyObject) -> PyResult<String> {
        let isi = if let Ok(isi) = isi.extract::<PyInstruction>(py) {
            vec![isi.0]
        } else if let Ok(isi) = isi.extract::<Vec<PyInstruction>>(py) {
            isi.into_iter().map(|isi| isi.0).collect()
        } else {
            return Err(PyValueError::new_err(""));
        };
        self.client
            .submit_all(isi)
            .map(|hash| hash.to_string())
            .map_err(|e| PyRuntimeError::new_err(format!("Error submitting instruction: {}", e)))
    }

    fn submit_executable_only_success(&self, py: Python<'_>, isi: PyObject) -> PyResult<String> {
        let isi = if let Ok(isi) = isi.extract::<PyInstruction>(py) {
            vec![isi.0]
        } else if let Ok(isi) = isi.extract::<Vec<PyInstruction>>(py) {
            isi.into_iter().map(|isi| isi.0).collect()
        } else {
            return Err(PyValueError::new_err(""));
        };

        let transaction = self.client.build_transaction(isi, Metadata::default());
        let hash = transaction.hash();
        self.client.submit_transaction(&transaction)?;

        let filters = vec![
            TransactionEventFilter::default().for_hash(hash).into(),
            PipelineEventFilterBox::from(
                BlockEventFilter::default().for_status(BlockStatus::Applied),
            ),
        ];

        let mut block_height: u64 = 0;
        for event in self.client.listen_for_events(filters)? {
            let event = event?;
            if let EventBox::Pipeline(event) = event {
                match event {
                    PipelineEventBox::Transaction(event) => {
                        if event.status == TransactionStatus::Approved
                            && event.block_height.is_some()
                        {
                            block_height = event.block_height.unwrap().get();
                        } else {
                            return Err(PyValueError::new_err("Transaction was not approved."));
                        }
                    }
                    PipelineEventBox::Block(event) => {
                        if event.header().height.get() == block_height {
                            return Ok(hash.to_string());
                        }
                    }
                }
            }
        }
        Err(PyValueError::new_err("No events left."))
    }

    fn query_all_domains(&self) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::domain::FindDomains)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.id.to_string());
        }
        Ok(items)
    }

    fn query_all_accounts(&self) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::account::FindAccounts)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.id.to_string());
        }
        Ok(items)
    }

    fn query_all_accounts_in_domain(&self, domain_id: &str) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::account::FindAccounts)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            let string = item.id.to_string();
            if string.len() > domain_id.len() + 1 {
                let string = &string[string.len() - domain_id.len() - 1..];
                if &string[0..1] == "@" && &string[1..] == domain_id {
                    items.push(item.id.to_string());
                }
            }
        }
        Ok(items)
    }

    fn query_all_assets(&self) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::asset::FindAssets)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.id.to_string());
        }
        Ok(items)
    }

    fn query_all_assets_owned_by_account(&self, account_id: &str) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::asset::FindAssets)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            let string = item.id.to_string();
            if string.len() > account_id.len() + 1 {
                let string = &string[string.len() - account_id.len() - 1..];
                if &string[0..1] == "#" && &string[1..] == account_id {
                    items.push(item.id.to_string());
                }
            }
        }
        Ok(items)
    }

    fn query_all_asset_definitions(&self) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::asset::FindAssetsDefinitions)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.id.to_string());
        }
        Ok(items)
    }

    fn query_all_block_headers(&self) -> PyResult<Vec<PyBlockHeader>> {
        let val = self
            .client
            .query(query::block::FindBlockHeaders)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.into());
        }
        Ok(items)
    }

    fn query_all_roles(&self) -> PyResult<Vec<PyRole>> {
        let val = self
            .client
            .query(query::role::FindRoles)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.into());
        }
        Ok(items)
    }

    fn query_all_role_ids(&self) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::role::FindRoleIds)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.to_string());
        }
        Ok(items)
    }

    fn query_all_roles_of_account(&self, account_id: &str) -> PyResult<Vec<String>> {
        let val = self
            .client
            .query(query::role::FindRolesByAccountId {
                id: AccountId::from_str(account_id)
                    .map_err(|e| PyValueError::new_err(e.to_string()))?,
            })
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.to_string());
        }
        Ok(items)
    }

    fn query_all_transactions(&self) -> PyResult<Vec<PyTransactionQueryOutput>> {
        let val = self
            .client
            .query(query::transaction::FindTransactions)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(item.into());
        }
        Ok(items)
    }

    fn query_all_transactions_by_account(
        &self,
        account_id: &str,
    ) -> PyResult<Vec<PyTransactionQueryOutput>> {
        let val = self
            .client
            .query(query::transaction::FindTransactions)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            if item.transaction.value.payload().authority.to_string() == account_id {
                items.push(item.into());
            }
        }
        Ok(items)
    }

    fn query_transaction_by_hash(
        &self,
        tx_hash: [u8; Hash::LENGTH],
    ) -> PyResult<PyTransactionQueryOutput> {
        let val = self
            .client
            .query(query::transaction::FindTransactions)
            .execute_all()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        for item in val {
            if item.transaction.value.hash()
                == HashOf::from_untyped_unchecked(Hash::prehashed(tx_hash))
            {
                return Ok(item.into());
            }
        }
        Err(PyRuntimeError::new_err("Transaction not found."))
    }
}

pub fn register_items(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Client>()?;
    Ok(())
}
