use std::{
    ffi::{c_char, c_int, CStr},
    future::IntoFuture,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::slice_from_raw_parts,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::collections::HashMap as StdHashMap;
use std::sync::{Arc, Mutex};
use surrealdb_core::dbs::Session;
use surrealdb_core::kvs::Datastore;
use surrealdb_core::rpc::format::cbor;
use surrealdb_core::rpc::RpcProtocol;
use surrealdb_rpc::error::{invalid_request, parse_error};
use surrealdb_rpc::{DbResponse, DbResult, Request};
use surrealdb::types::{Action, Notification};
use async_channel::Receiver;
use surrealdb::types::SurrealValue;
use surrealdb::types::{Value as sdbValue, HashMap};
use tokio::{runtime::Runtime, sync::RwLock};

use crate::{array::MakeArray, opts::Options, stream::RpcStream, string::string_t, SR_ERROR, SR_FATAL};

/// The object representing a Surreal RPC connection
///
/// It is safe to be referenced from multiple threads
/// If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
/// (use will cause the program to abort)
///
/// should be freed with sr_surreal_rpc_free
pub struct SurrealRpc {
    inner: RwLock<SurrealRpcInner>,
    rt: Runtime,
    ps: AtomicBool,
}
/// create new rpc context
///
/// # Examples
///
/// ```c
/// sr_string_t err;
/// sr_surreal_rpc_t ctx;
///
/// sr_surreal_rpc_new(err, ctx, "surrealkv://test.db", {});
///
/// ```
impl SurrealRpc {
    #[export_name = "sr_surreal_rpc_new"]
    pub extern "C" fn new(
        err_ptr: *mut string_t,
        surreal_ptr: *mut *mut SurrealRpc,
        endpoint: *const c_char,
        options: Options,
    ) -> c_int {
        let res: Result<Result<SurrealRpc, string_t>, _> = catch_unwind(AssertUnwindSafe(|| {
            let Ok(endpoint) = (unsafe { CStr::from_ptr(endpoint).to_str() }) else {
                return Err("Invalid UTF-8".into());
            };

            let Ok(rt) = Runtime::new() else {
                return Err("error creating runtime".into());
            };

            let (notify_tx, datastore_notifications) = async_channel::bounded(LQ_CHANNEL_SIZE);
            let mut builder = Datastore::builder().with_notify(notify_tx);
            if options.query_timeout != 0 {
                builder = builder
                    .with_query_timeout(Some(Duration::from_secs(options.query_timeout as u64)))
            }
            if options.transaction_timeout != 0 {
                builder = builder.with_transaction_timeout(Some(Duration::from_secs(
                    options.transaction_timeout as u64,
                )))
            }

            // Built inside the runtime: the datastore starts its background tasks here
            let kvs = match rt.block_on(builder.build_with_path(endpoint).into_future()) {
                Ok(db) => db,
                Err(e) => return Err(e.to_string().into()),
            };

            // Requests without a `session` field use this connection-level session,
            // as they do on a server WebSocket connection
            let default_session = uuid::Uuid::new_v4();
            let mut session = Session::default().with_rt(true);
            session.id = Some(default_session);
            let session_map = HashMap::default();
            session_map.insert(default_session, Arc::new(RwLock::new(session)));

            // Forward notifications to sr_surreal_rpc_notifications streams with the session
            // that started the live query (none for the default session), forgetting each live
            // query once it ends (by KILL, or when its table is removed)
            let live_queries = LiveQueries::default();
            let (forward_tx, notifications) = async_channel::bounded(LQ_CHANNEL_SIZE);
            let lqs = live_queries.clone();
            rt.spawn(async move {
                while let Ok(notification) = datastore_notifications.recv().await {
                    let session = {
                        let mut lqs = lqs.lock().unwrap_or_else(|e| e.into_inner());
                        let id = uuid::Uuid::from(notification.id);
                        if notification.action == Action::Killed {
                            lqs.remove(&id)
                        } else {
                            lqs.get(&id).copied()
                        }
                    };
                    let session = session
                        .or(notification.session.map(Into::into))
                        .filter(|s| *s != default_session);
                    if forward_tx.send((notification, session)).await.is_err() {
                        break;
                    }
                }
            });

            let inner = SurrealRpcInner {
                kvs,
                session_map,
                default_session,
                notifications,
                live_queries,
            };

            Ok(SurrealRpc {
                inner: RwLock::new(inner),
                rt,
                ps: AtomicBool::new(false),
            })
        }));

        let res: Result<SurrealRpc, string_t> = match res {
            Ok(r) => r,
            Err(e) => {
                if let Some(e_str) = e.downcast_ref::<&str>() {
                    let e_string: string_t = format!("Panicked with: {e_str}").into();
                    unsafe { err_ptr.write(e_string) }
                } else {
                    unsafe { err_ptr.write("Panicked".into()) }
                }
                return SR_FATAL;
            }
        };

        match res {
            Ok(s) => {
                let boxed = Box::new(s);
                unsafe { surreal_ptr.write(Box::leak(boxed)) }
                1
            }
            Err(e) => {
                unsafe { err_ptr.write(e) }
                SR_ERROR
            }
        }
    }

    /// Execute an RPC request via raw CBOR bytes
    ///
    /// `ptr`/`len` hold one CBOR-encoded request in the same format the server's WebSocket
    /// endpoint accepts: a map with `method`, optional `params`, and optional `id`, `session`
    /// and `txn`. Values use SurrealDB's CBOR tags (record ids, datetimes, uuids, ...).
    ///
    /// On success, `*res_ptr` receives one CBOR-encoded response in the server's format:
    /// `{ id, result }`, or `{ id, error }` if the method failed. The return value is the
    /// length of that buffer. A request that cannot be decoded also produces an `error`
    /// response, as it would from the server.
    ///
    /// Returns a negative code, and writes no response, only for invalid arguments (null
    /// pointers, negative length) or a fatal error.
    ///
    /// # Safety
    ///
    /// - `ctx` must be a pointer returned by `sr_surreal_rpc_new`
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result
    /// - `ptr` must be a valid pointer to `len` bytes of CBOR-encoded request data
    ///
    /// The response buffer is owned by the caller; free it with `sr_free_byte_arr(*res_ptr, len)`.
    #[export_name = "sr_surreal_rpc_execute"]
    pub extern "C" fn execute(
        &self,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut u8,
        ptr: *const u8,
        len: c_int,
    ) -> c_int {
        if res_ptr.is_null() {
            if !err_ptr.is_null() {
                unsafe { err_ptr.write("res_ptr is null".into()) };
            }
            return SR_ERROR;
        }
        if ptr.is_null() || len < 0 {
            if !err_ptr.is_null() {
                unsafe { err_ptr.write("ptr is null or len is negative".into()) };
            }
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let in_bytes = unsafe { &*slice_from_raw_parts(ptr, len as usize) };

            let inner = ctx.inner.read().await;
            let response = inner.handle(in_bytes).await;

            let out_bytes = cbor::encode(response.into_value())
                .map_err(|e| string_t::from(format!("CBOR encode error: {e}")))?;
            let out = out_bytes.make_array();
            unsafe { res_ptr.write(out.ptr) }
            Ok(out.len)
        })
    }

    /// Get a stream for receiving live query notifications
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `stream_ptr` must be a valid pointer to receive the stream
    ///
    /// Returns a stream that can be polled for notifications using sr_rpc_stream_next
    #[export_name = "sr_surreal_rpc_notifications"]
    pub extern "C" fn notifications(
        &self,
        err_ptr: *mut string_t,
        stream_ptr: *mut *mut RpcStream,
    ) -> c_int {
        if stream_ptr.is_null() {
            if !err_ptr.is_null() {
                unsafe { err_ptr.write("stream_ptr is null".into()) };
            }
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let receiver = ctx.inner.read().await.notifications.clone();

            let rpc_stream = RpcStream::new(receiver);
            let stream_boxed = Box::new(rpc_stream);
            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };

            Ok(1)
        })
    }

    /// Free an RPC context
    #[export_name = "sr_surreal_rpc_free"]
    pub extern "C" fn rpc_free(ctx: *mut SurrealRpc) {
        if ctx.is_null() {
            return;
        }
        let boxed = unsafe { Box::from_raw(ctx) };
        // End this context's live queries, so a persistent datastore doesn't keep them
        let _ = catch_unwind(AssertUnwindSafe(|| {
            boxed.rt.block_on(async { boxed.inner.read().await.cleanup_all_lqs().await })
        }));
        drop(boxed)
    }
}

fn with_async<'a, 'b, C, F>(ctx: &'a SurrealRpc, err_ptr: *mut string_t, fun: C) -> c_int
where
    'a: 'b,
    C: FnOnce(&'a SurrealRpc) -> F + 'b,
    F: std::future::Future<Output = Result<c_int, string_t>>,
{
    if ctx.ps.load(Ordering::Acquire) {
        std::process::abort()
    }
    let _guard = ctx.rt.enter();

    let res = match catch_unwind(AssertUnwindSafe(|| ctx.rt.block_on(fun(&ctx)))) {
        Ok(r) => r,
        Err(e) => {
            if let Some(e_str) = e.downcast_ref::<&str>() {
                let e_string: string_t = format!("Panicked with: {e_str}").into();
                unsafe { err_ptr.write(e_string) }
            } else {
                unsafe { err_ptr.write("Panicked".into()) }
            }
            return SR_FATAL;
        }
    };

    match res {
        Ok(n) => n,
        Err(e) => {
            unsafe { err_ptr.write(e) }
            SR_ERROR
        }
    }
}

/// Maximum nesting depth of a decoded request (the limit core's `encoding::cbor::decode` uses)
const CBOR_RECURSION_LIMIT: usize = 100;

/// Capacity of the notification channel (the size the 3.0 datastore used)
const LQ_CHANNEL_SIZE: usize = 15_000;

/// The session each running live query was started from
type LiveQueries = Arc<Mutex<StdHashMap<uuid::Uuid, uuid::Uuid>>>;

struct SurrealRpcInner {
    kvs: Arc<Datastore>,
    session_map: HashMap<uuid::Uuid, Arc<RwLock<Session>>>,
    default_session: uuid::Uuid,
    notifications: Receiver<(Notification, Option<uuid::Uuid>)>,
    /// The session each running live query was started from, so detaching or resetting a
    /// session (or freeing the context) can end its live queries
    live_queries: LiveQueries,
}

impl SurrealRpcInner {
    /// Remove the matching live queries from the datastore
    async fn kill_lqs(&self, matches: impl Fn(&uuid::Uuid) -> bool) {
        let ids: Vec<uuid::Uuid> = {
            let mut lqs = self.live_queries.lock().unwrap_or_else(|e| e.into_inner());
            let ids = lqs.iter().filter(|(_, s)| matches(s)).map(|(lq, _)| *lq).collect();
            lqs.retain(|_, s| !matches(s));
            ids
        };
        // A query already ended by KILL is skipped by delete_queries
        if !ids.is_empty() {
            let _ = self.kvs.delete_queries(ids).await;
        }
    }

    /// Decode and execute one CBOR request, returning the response the server would send
    async fn handle(&self, bytes: &[u8]) -> DbResponse {
        let request = match cbor::decode(bytes, CBOR_RECURSION_LIMIT) {
            Ok(sdbValue::Object(obj)) => Request::from_object(obj),
            Ok(_) => Err(invalid_request()),
            Err(_) => Err(parse_error()),
        };
        let request = match request {
            Ok(request) => request,
            Err(e) => return DbResponse::failure(None, None, e),
        };
        let txn = request.txn.map(Into::into);
        let client_session: Option<uuid::Uuid> = request.session_id.map(Into::into);
        let session = client_session.unwrap_or(self.default_session);
        let result =
            self.execute(txn, session, client_session, request.method, request.params).await;
        DbResponse::new(request.id, client_session, result)
    }
}

impl RpcProtocol for SurrealRpcInner {
    fn kvs(&self) -> &Datastore {
        &self.kvs
    }

    fn kvs_arc(&self) -> Arc<Datastore> {
        self.kvs.clone()
    }

    fn session_map(&self) -> &HashMap<uuid::Uuid, Arc<RwLock<Session>>> {
        &self.session_map
    }

    fn version_data(&self) -> DbResult {
        let ver_str = surrealdb_core::env::VERSION.to_string();
        DbResult::Other(sdbValue::String(ver_str))
    }
    
    const LQ_SUPPORT: bool = true;

    async fn handle_live(
        &self,
        lqid: &uuid::Uuid,
        session_id: uuid::Uuid,
        _namespace: Option<String>,
        _database: Option<String>,
    ) {
        self.live_queries.lock().unwrap_or_else(|e| e.into_inner()).insert(*lqid, session_id);
    }

    async fn cleanup_lqs(&self, session_id: &uuid::Uuid) {
        self.kill_lqs(|s| s == session_id).await
    }

    async fn cleanup_all_lqs(&self) {
        self.kill_lqs(|_| true).await
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use surrealdb::types::{Array, Object};

    use super::*;

    fn new_ctx() -> Box<SurrealRpc> {
        let mut err = string_t::null();
        let mut ctx: *mut SurrealRpc = std::ptr::null_mut();
        let endpoint = CString::new("memory").unwrap();
        let opts = Options { strict: false, query_timeout: 0, transaction_timeout: 0 };
        assert!(SurrealRpc::new(&mut err, &mut ctx, endpoint.as_ptr(), opts) > 0);
        unsafe { Box::from_raw(ctx) }
    }

    /// Run one request and return its result
    fn call(ctx: &SurrealRpc, method: &str, params: Vec<sdbValue>) -> sdbValue {
        let mut req = Object::new();
        req.insert("method".to_string(), sdbValue::String(method.to_string()));
        req.insert("params".to_string(), sdbValue::Array(Array::from(params)));
        let bytes = cbor::encode(sdbValue::Object(req)).unwrap();
        let response = ctx.rt.block_on(async { ctx.inner.read().await.handle(&bytes).await });
        let sdbValue::Object(mut response) = response.into_value() else { panic!("not an object") };
        response.remove("result").unwrap_or_else(|| panic!("{method} failed: {response:?}"))
    }

    fn tracked(ctx: &SurrealRpc) -> usize {
        ctx.rt.block_on(async { ctx.inner.read().await.live_queries.lock().unwrap().len() })
    }

    #[test]
    fn killed_live_queries_are_forgotten() {
        let ctx = new_ctx();
        let s = |v: &str| sdbValue::String(v.to_string());
        call(&ctx, "use", vec![s("test"), s("test")]);
        call(&ctx, "query", vec![s("DEFINE TABLE person")]);

        let id = call(&ctx, "live", vec![s("person")]);
        assert!(matches!(id, sdbValue::Uuid(_)), "live returns a uuid: {id:?}");
        assert_eq!(tracked(&ctx), 1);

        call(&ctx, "kill", vec![id]);
        // The KILLED notification is forwarded asynchronously
        for _ in 0..100 {
            if tracked(&ctx) == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(tracked(&ctx), 0, "the killed live query is still tracked");
    }
}
