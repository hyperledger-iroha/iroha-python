use paste::paste;

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

use iroha::client::Client as IrohaClient;
use iroha::config::Config as IrohaClientConfig;
use iroha::config::{BasicAuth, WebLogin};

use std::num::NonZeroU64;
use std::str::FromStr;

use crate::data_model::asset::{PyAsset, PyAssetDefinition, PyAssetDefinitionId, PyAssetId};
use crate::data_model::block::*;
use crate::data_model::crypto::*;
use crate::data_model::role::*;
use crate::data_model::tx::*;
use crate::data_model::PyMirror;
use crate::{data_model::account::PyAccountId, isi::PyInstruction};
use iroha_crypto::{Hash, HashOf};
use iroha_data_model::account::AccountId;
use iroha_data_model::prelude::*;
use iroha_data_model::ChainId;

use iroha_data_model::events::pipeline::{BlockEventFilter, TransactionEventFilter};

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

    fn query_transaction_with_hash(&self, hash: [u8; Hash::LENGTH]) -> PyResult<bool> {
        let query = iroha_data_model::query::prelude::FindTransactionByHash {
            hash: HashOf::from_untyped_unchecked(Hash::prehashed(hash)).into(),
        };

        let ret = self.client.request(query);
        println!("{:?}", ret);
        Ok(ret
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .is_ok())
    }

    fn query_all_domains(&self) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAllDomains {};

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_accounts_in_domain(&self, domain_id: &str) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAccountsByDomainId {
            domain: DomainId::from_str(domain_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .into(),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_accounts(&self) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAllAccounts;

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_assets_owned_by_account(&self, account_id: &str) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAssetsByAccountId {
            account: AccountId::from_str(account_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .into(),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_assets(&self) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAllAssets;

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_asset_definitions(&self) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAllAssetsDefinitions;

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.id.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_block_headers(&self) -> PyResult<Vec<PyBlockHeader>> {
        let query = iroha_data_model::query::prelude::FindAllBlockHeaders;

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.into())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_roles(&self) -> PyResult<Vec<PyRole>> {
        let query = iroha_data_model::query::prelude::FindAllRoles {};

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.into())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_role_ids(&self) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindAllRoleIds {};

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_role_by_id(&self, role_id: &str) -> PyResult<PyRole> {
        let query = iroha_data_model::query::prelude::FindRoleByRoleId {
            id: RoleId::from_str(role_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .into(),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        Ok(val.into())
    }

    fn query_all_roles_of_account(&self, account_id: &str) -> PyResult<Vec<String>> {
        let query = iroha_data_model::query::prelude::FindRolesByAccountId {
            id: AccountId::from_str(account_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .into(),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.to_string())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_transactions(&self) -> PyResult<Vec<PyTransactionQueryOutput>> {
        let query = iroha_data_model::query::prelude::FindAllTransactions;

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.into())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_all_transactions_by_account(
        &self,
        account_id: &str,
    ) -> PyResult<Vec<PyTransactionQueryOutput>> {
        let query = iroha_data_model::query::prelude::FindTransactionsByAccountId {
            account: AccountId::from_str(account_id)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .into(),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        let mut items = Vec::new();
        for item in val {
            items.push(
                item.map(|d| d.into())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?,
            );
        }
        Ok(items)
    }

    fn query_transaction_by_hash(
        &self,
        tx_hash: [u8; Hash::LENGTH],
    ) -> PyResult<PyTransactionQueryOutput> {
        let query = iroha_data_model::query::prelude::FindTransactionByHash {
            hash: HashOf::from_untyped_unchecked(Hash::prehashed(tx_hash).into()),
        };

        let val = self
            .client
            .request(query)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;

        Ok(val.into())
    }
}

macro_rules! register_query {
    ($query_name:ty; $ret:ty) => {
        register_query!($query_name; $ret;);
    };
    ($query_name:ty; $ret:ty; $($param_name:ident: $param_typ:ty),*) => {
        paste! {
            #[pymethods]
            impl Client {
                fn [<$query_name:snake>](
                    &self,
                    $($param_name: $param_typ),*
                ) -> PyResult<$ret> {
                    #[allow(unused_imports)]
                    use std::ops::Deref as _;

                    let query = iroha_data_model::query::prelude::$query_name {
                        $(
                            $param_name: $param_name.deref().clone().into()
                        ),*
                    };
                    let val = self.client.request(query)
                        .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
                    val.mirror()
                }
            }
        }
    };
}

pub fn register_items(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<Client>()?;
    Ok(())
}
