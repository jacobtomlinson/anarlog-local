#![forbid(unsafe_code)]

mod db;
mod error;
mod listener;

use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};

use error::{BridgeError, execute_error, parse_params_json, reactive_error, serialization_error};
use listener::{ListenerSink, QueryEventListener};

uniffi::setup_scaffolding!();

fn block_on<F: Future>(runtime: &tokio::runtime::Runtime, future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| runtime.handle().block_on(future))
        }
        _ => runtime.handle().block_on(future),
    }
}

struct BridgeState {
    db: Arc<anlg_db_core::Db>,
    executor: anlg_db_execute::DbExecutor,
    live_query_runtime: Arc<anlg_db_reactive::LiveQueryRuntime<ListenerSink>>,
    runtime: Arc<tokio::runtime::Runtime>,
    subscription_ids: HashSet<String>,
}

#[derive(uniffi::Object)]
pub struct MobileDbBridge {
    state: Mutex<Option<BridgeState>>,
}

#[uniffi::export]
impl MobileDbBridge {
    #[uniffi::constructor]
    pub fn open(db_path: String) -> Result<Self, BridgeError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| BridgeError::OpenFailed {
                reason: error.to_string(),
            })?;
        let runtime = Arc::new(runtime);
        let path = std::path::PathBuf::from(db_path);
        let db = block_on(&runtime, db::open_app_db(&path)).map_err(|error| {
            BridgeError::OpenFailed {
                reason: error.to_string(),
            }
        })?;
        let db = Arc::new(db);
        let executor = anlg_db_execute::DbExecutor::new(Arc::clone(&db));
        let live_query_runtime = {
            let _guard = runtime.enter();
            Arc::new(anlg_db_reactive::LiveQueryRuntime::new(Arc::clone(&db)))
        };

        Ok(Self {
            state: Mutex::new(Some(BridgeState {
                db,
                executor,
                live_query_runtime,
                runtime,
                subscription_ids: HashSet::new(),
            })),
        })
    }

    pub fn execute(&self, sql: String, params_json: String) -> Result<String, BridgeError> {
        let params = parse_params_json(&params_json)?;
        let (runtime, executor) =
            self.with_state(|state| Ok((Arc::clone(&state.runtime), state.executor.clone())))?;
        let rows = block_on(&runtime, executor.execute(sql, params)).map_err(execute_error)?;
        serde_json::to_string(&rows).map_err(serialization_error)
    }

    pub fn execute_proxy(
        &self,
        sql: String,
        params_json: String,
        method: String,
    ) -> Result<String, BridgeError> {
        let params = parse_params_json(&params_json)?;
        let method = method
            .parse::<anlg_db_execute::ProxyQueryMethod>()
            .map_err(execute_error)?;
        let (runtime, executor) =
            self.with_state(|state| Ok((Arc::clone(&state.runtime), state.executor.clone())))?;
        let rows = block_on(&runtime, executor.execute_proxy(sql, params, method))
            .map_err(execute_error)?;
        serde_json::to_string(&rows).map_err(serialization_error)
    }

    pub fn execute_transaction(&self, statements_json: String) -> Result<String, BridgeError> {
        let statements: Vec<anlg_db_execute::TransactionStatement> =
            serde_json::from_str(&statements_json).map_err(|error| {
                BridgeError::InvalidTransactionStatementsJson {
                    reason: error.to_string(),
                }
            })?;
        let (runtime, executor) =
            self.with_state(|state| Ok((Arc::clone(&state.runtime), state.executor.clone())))?;
        let rows_affected =
            block_on(&runtime, executor.execute_transaction(statements)).map_err(execute_error)?;
        serde_json::to_string(&rows_affected).map_err(serialization_error)
    }

    pub fn subscribe(
        &self,
        sql: String,
        params_json: String,
        listener: Arc<dyn QueryEventListener>,
    ) -> Result<String, BridgeError> {
        let params = parse_params_json(&params_json)?;
        let (runtime, live_query_runtime) = self.with_state(|state| {
            Ok((
                Arc::clone(&state.runtime),
                Arc::clone(&state.live_query_runtime),
            ))
        })?;
        let registration = block_on(
            &runtime,
            live_query_runtime.subscribe(sql, params, ListenerSink::new(listener)),
        )
        .map_err(reactive_error)?;
        let subscription_id = registration.id.clone();
        if self
            .with_state(|state| {
                state.subscription_ids.insert(subscription_id);
                Ok(())
            })
            .is_err()
        {
            let _ = block_on(&runtime, live_query_runtime.unsubscribe(&registration.id));
            return Err(BridgeError::Closed);
        }
        Ok(registration.id)
    }

    pub fn unsubscribe(&self, subscription_id: String) -> Result<(), BridgeError> {
        let (runtime, live_query_runtime) = self.with_state(|state| {
            Ok((
                Arc::clone(&state.runtime),
                Arc::clone(&state.live_query_runtime),
            ))
        })?;
        block_on(&runtime, live_query_runtime.unsubscribe(&subscription_id))
            .map_err(reactive_error)?;
        self.with_state(|state| {
            state.subscription_ids.remove(&subscription_id);
            Ok(())
        })
    }

    pub fn close(&self) -> Result<(), BridgeError> {
        let mut guard = self.state.lock().unwrap();
        let Some(mut state) = guard.take() else {
            return Ok(());
        };
        drop(guard);
        let subscription_ids: Vec<String> = state.subscription_ids.drain().collect();
        let pool = state.db.pool().clone();
        block_on(&state.runtime, async {
            for subscription_id in subscription_ids {
                let _ = state.live_query_runtime.unsubscribe(&subscription_id).await;
            }
        });
        drop(state.live_query_runtime);
        drop(state.executor);
        drop(state.db);
        block_on(&state.runtime, pool.close());
        Ok(())
    }
}

impl MobileDbBridge {
    fn with_state<T>(
        &self,
        f: impl FnOnce(&mut BridgeState) -> Result<T, BridgeError>,
    ) -> Result<T, BridgeError> {
        let mut guard = self.state.lock().unwrap();
        let state = guard.as_mut().ok_or(BridgeError::Closed)?;
        f(state)
    }
}

impl Drop for MobileDbBridge {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
