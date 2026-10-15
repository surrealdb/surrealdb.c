use std::{
    ffi::{c_char, c_int, CStr},
    future::IntoFuture,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::slice_from_raw_parts,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::Arc;
use surrealdb_core::dbs::Session;
use surrealdb_core::kvs::Datastore;
use surrealdb_core::rpc::format::cbor;
use surrealdb_core::rpc::{
    invalid_request, parse_error, DbResponse, DbResult, Request, RpcProtocol,
};
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

            let con_fut = Datastore::new(endpoint);

            let mut kvs = match rt.block_on(con_fut.into_future()) {
                Ok(db) => db,
                Err(e) => return Err(e.to_string().into()),
            };

            kvs = kvs.with_notifications();

            if options.query_timeout != 0 {
                kvs =
                    kvs.with_query_timeout(Some(Duration::from_secs(options.query_timeout as u64)))
            }
            if options.transaction_timeout != 0 {
                kvs = kvs.with_transaction_timeout(Some(Duration::from_secs(
                    options.transaction_timeout as u64,
                )))
            }

            let session_map = HashMap::default();
            let default_session = Arc::new(RwLock::new(Session::default().with_rt(true)));
            session_map.insert(None, default_session);

            let inner = SurrealRpcInner {
                kvs,
                session_map,
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
            let receiver = ctx
                .inner
                .read()
                .await
                .kvs
                .notifications()
                .ok_or(string_t::from("Notifications not enabled"))?;

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

#[allow(dead_code)]
struct SurrealRpcInner {
    kvs: Datastore,
    session_map: HashMap<Option<uuid::Uuid>, Arc<RwLock<Session>>>,
}

impl SurrealRpcInner {
    /// Decode and execute one CBOR request, returning the response the server would send
    async fn handle(&self, bytes: &[u8]) -> DbResponse {
        let request = match cbor::decode(bytes) {
            Ok(sdbValue::Object(obj)) => Request::from_object(obj),
            Ok(_) => Err(invalid_request()),
            Err(_) => Err(parse_error()),
        };
        let request = match request {
            Ok(request) => request,
            Err(e) => return DbResponse::failure(None, None, e),
        };
        let txn = request.txn.map(Into::into);
        let session = request.session_id.map(Into::into);
        let result = self.execute(txn, session, request.method, request.params).await;
        DbResponse::new(request.id, session, result)
    }
}

impl RpcProtocol for SurrealRpcInner {
    fn kvs(&self) -> &Datastore {
        &self.kvs
    }

    fn session_map(&self) -> &HashMap<Option<uuid::Uuid>, Arc<RwLock<Session>>> {
        &self.session_map
    }

    fn version_data(&self) -> DbResult {
        let ver_str = surrealdb_core::env::VERSION.to_string();
        DbResult::Other(sdbValue::String(ver_str))
    }
    
    const LQ_SUPPORT: bool = true;

    async fn handle_live(&self, _lqid: &uuid::Uuid, _session_id: Option<uuid::Uuid>) {}

    async fn handle_kill(&self, _lqid: &uuid::Uuid) {}

    async fn cleanup_lqs(&self, _session_id: Option<&uuid::Uuid>) {}

    async fn cleanup_all_lqs(&self) {}
}
