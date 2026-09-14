use std::{
    ffi::{c_char, c_int, CStr},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::slice_from_raw_parts,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::Arc;
use async_channel::Receiver;
use surrealdb_core::dbs::Session;
use surrealdb_core::kvs::{Builder, Datastore};
use surrealdb_core::rpc::{Method, RpcProtocol, DbResult};
use surrealdb::types::{Value as sdbValue, HashMap, Notification as PublicNotification, SurrealValue};
use tokio::{runtime::Runtime, sync::RwLock};

use crate::{array::{ArrayGen, MakeArray}, opts::Options, stream::RpcStream, string::string_t, uuid::Uuid, write_error, SR_ERROR, SR_FATAL};

/// The object representing a Surreal RPC connection
///
/// It is safe to be referenced from multiple threads
/// If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
/// (use will cause the program to abort)
///
/// should be freed with sr_surreal_rpc_disconnect
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

            // As of SurrealDB 3.1, the caller owns the notification channel: the
            // datastore no longer hands one back via `Datastore::notifications()`.
            let (notify_tx, notify_rx) = async_channel::unbounded::<PublicNotification>();

            let mut builder = Builder::new().with_notify(notify_tx);

            // Capabilities are a sandbox, so the builder is only touched when
            // the caller actually asked for something. A parse failure is an
            // error rather than a silent skip: dropping a name would change the
            // sandbox without telling anyone.
            //
            // Experimental features are gated twice -- by a Cargo feature at
            // compile time and by the capability here. With only the feature
            // you get "Experimental capability `gql` is not enabled" at the
            // point of use, which is a confusing place to find out.
            match options.capabilities.to_capabilities() {
                Ok(Some(caps)) => builder = builder.with_capabilities(caps),
                Ok(None) => {}
                Err(e) => return Err(string_t::from(e)),
            }

            if options.query_timeout != 0 {
                builder = builder
                    .with_query_timeout(Some(Duration::from_secs(options.query_timeout as u64)))
            }
            if options.transaction_timeout != 0 {
                builder = builder.with_transaction_timeout(Some(Duration::from_secs(
                    options.transaction_timeout as u64,
                )))
            }

            let kvs = match rt.block_on(builder.build_with_path(endpoint)) {
                Ok(db) => Arc::new(db),
                Err(e) => return Err(e.to_string().into()),
            };

            // Sessions are keyed by a real `Uuid` as of 3.1 — there is deliberately
            // no type-level "default session" any more (GHSA-4vgr-h27g-cf9p), so we
            // mint one and hold onto its id.
            let default_session_id = uuid::Uuid::new_v4();
            let session_map = HashMap::default();
            session_map.insert(
                default_session_id,
                Arc::new(RwLock::new(Session::default().with_rt(true))),
            );

            let inner = SurrealRpcInner {
                kvs,
                session_map,
                live_queries: HashMap::new(),
                notify_rx,
                default_session: default_session_id,
                session_dir: options.session_dir_path(),
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
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result
    /// - `ptr` must be a valid pointer to CBOR-encoded request data
    /// - `len` must be the length of the data at ptr
    ///
    /// Free result with sr_byte_arr_free
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
        if ptr.is_null() {
            if !err_ptr.is_null() {
                unsafe { err_ptr.write("ptr is null".into()) };
            }
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let inner = ctx.inner.read().await;
            let session = inner.default_session;
            run_rpc(&*inner, session, res_ptr, ptr, len).await
        })
    }

    // ------------------------------------------------------------------
    // Sessions
    // ------------------------------------------------------------------
    //
    // SurrealDB 3.1 removed the type-level "default session": every request
    // names a session explicitly (GHSA-4vgr-h27g-cf9p). This context mints one
    // at construction so the simple `sr_surreal_rpc_execute` path keeps
    // working, and the functions below expose the rest of the model.
    //
    // A session id is a plain `sr_uuid_t` and carries no ownership -- sessions
    // live in the context and are released by `sr_rpc_session_detach` or by
    // disconnecting the context.

    /// Register a new session.
    ///
    /// Pass an all-zero `session_id` to have one generated and written back;
    /// otherwise the supplied id is used. Attaching an id that already exists
    /// is an error.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `session_id` must be a valid pointer to a 16-byte uuid
    #[export_name = "sr_rpc_session_attach"]
    pub extern "C" fn session_attach(
        &self,
        err_ptr: *mut string_t,
        session_id: *mut Uuid,
    ) -> c_int {
        if session_id.is_null() {
            write_error(err_ptr, "session_id is null");
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let requested = unsafe { (*session_id).clone() };
            let id = if requested.0 == [0u8; 16] {
                uuid::Uuid::new_v4()
            } else {
                uuid::Uuid::from(requested)
            };

            let inner = ctx.inner.read().await;
            <SurrealRpcInner as RpcProtocol>::attach(&*inner, id)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            unsafe { session_id.write(Uuid::from(id)) };
            Ok(1)
        })
    }

    /// Close a session, cancelling the live queries it owns.
    ///
    /// Each cancelled live query emits one final notification on the stream
    /// from `sr_surreal_rpc_notifications`, with action `KILLED` and no result.
    /// That is the signal that a live query is over; nothing further arrives
    /// for it.
    ///
    /// The durable copy of the session is removed first, so a detached session
    /// cannot be rehydrated. There are no client-managed transactions on this
    /// transport, so there are none to cancel.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `session_id` must be a valid pointer to a 16-byte uuid
    #[export_name = "sr_rpc_session_detach"]
    pub extern "C" fn session_detach(
        &self,
        err_ptr: *mut string_t,
        session_id: *const Uuid,
    ) -> c_int {
        if session_id.is_null() {
            write_error(err_ptr, "session_id is null");
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let id = uuid::Uuid::from(unsafe { (*session_id).clone() });
            let inner = ctx.inner.read().await;
            <SurrealRpcInner as RpcProtocol>::detach(&*inner, id)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Return a session to its initial state without closing it.
    ///
    /// This also cancels the session's live queries -- a reset drops the
    /// identity they were registered under, so they must not keep delivering.
    /// Each emits a final `KILLED` notification, as with
    /// `sr_rpc_session_detach`. The same applies to the authentication methods
    /// reached over `sr_surreal_rpc_execute`: `signin`, `signup`,
    /// `authenticate`, `refresh` and `invalidate` all retire the session's live
    /// queries, because the caller they were authorised for no longer exists.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `session_id` must be a valid pointer to a 16-byte uuid
    #[export_name = "sr_rpc_session_reset"]
    pub extern "C" fn session_reset(
        &self,
        err_ptr: *mut string_t,
        session_id: *const Uuid,
    ) -> c_int {
        if session_id.is_null() {
            write_error(err_ptr, "session_id is null");
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let id = uuid::Uuid::from(unsafe { (*session_id).clone() });
            let inner = ctx.inner.read().await;
            <SurrealRpcInner as RpcProtocol>::reset(&*inner, id)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// List the ids of every active session.
    ///
    /// Returns the count, and writes an array of that many uuids to
    /// `sessions_ptr`. Free it with `sr_uuid_arr_free`.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `sessions_ptr` must be a valid pointer to receive the array
    #[export_name = "sr_rpc_session_list"]
    pub extern "C" fn session_list(
        &self,
        err_ptr: *mut string_t,
        sessions_ptr: *mut *mut Uuid,
    ) -> c_int {
        if sessions_ptr.is_null() {
            write_error(err_ptr, "sessions_ptr is null");
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let inner = ctx.inner.read().await;
            let ids: Vec<Uuid> = inner
                .session_map
                .to_vec()
                .into_iter()
                .map(|(id, _)| Uuid::from(id))
                .collect();
            let out = ids.make_array();
            unsafe { sessions_ptr.write(out.ptr) };
            Ok(out.len)
        })
    }

    /// The id of the session this context created for itself, which
    /// `sr_surreal_rpc_execute` runs against.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `session_id` must be a valid pointer to receive a 16-byte uuid
    #[export_name = "sr_rpc_session_default"]
    pub extern "C" fn session_default(
        &self,
        err_ptr: *mut string_t,
        session_id: *mut Uuid,
    ) -> c_int {
        if session_id.is_null() {
            write_error(err_ptr, "session_id is null");
            return SR_ERROR;
        }
        with_async(self, err_ptr, |ctx| async {
            let inner = ctx.inner.read().await;
            unsafe { session_id.write(Uuid::from(inner.default_session)) };
            Ok(0)
        })
    }

    /// Execute an RPC request against a named session.
    ///
    /// Identical to `sr_surreal_rpc_execute` except that the session is chosen
    /// by the caller rather than defaulting to this context's own.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result
    /// - `session_id` must be a valid pointer to a 16-byte uuid
    /// - `ptr` must be a valid pointer to `len` bytes of CBOR request data
    ///
    /// Free the result with sr_byte_arr_free
    #[export_name = "sr_rpc_execute_on"]
    pub extern "C" fn execute_on(
        &self,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut u8,
        session_id: *const Uuid,
        ptr: *const u8,
        len: c_int,
    ) -> c_int {
        if res_ptr.is_null() {
            write_error(err_ptr, "res_ptr is null");
            return SR_ERROR;
        }
        if ptr.is_null() {
            write_error(err_ptr, "ptr is null");
            return SR_ERROR;
        }
        if session_id.is_null() {
            write_error(err_ptr, "session_id is null");
            return SR_ERROR;
        }
        let session = uuid::Uuid::from(unsafe { (*session_id).clone() });
        with_async(self, err_ptr, move |ctx| async move {
            let inner = ctx.inner.read().await;
            run_rpc(&*inner, session, res_ptr, ptr, len).await
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
            let receiver = ctx.inner.read().await.notify_rx.clone();

            let rpc_stream = RpcStream::new(receiver);
            let stream_boxed = Box::new(rpc_stream);
            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };

            Ok(1)
        })
    }

    /// Free an RPC context
    ///
    /// Also closes the notification channel, so any thread blocked in
    /// sr_rpc_stream_next returns SR_CLOSED. That is the supported way to shut
    /// a notification reader down. Streams obtained from this context stay
    /// valid afterwards and must still be freed with sr_rpc_stream_free.
    ///
    /// # Safety
    ///
    /// - `ctx` must be a valid pointer to a SurrealRpc, or null (no-op)
    #[export_name = "sr_surreal_rpc_disconnect"]
    pub extern "C" fn rpc_free(ctx: *mut SurrealRpc) {
        if ctx.is_null() {
            return;
        }
        let boxed = unsafe { Box::from_raw(ctx) };
        drop(boxed)
    }
}

fn parse_cbor_request(value: &ciborium::Value) -> Result<(String, surrealdb::types::Array), string_t> {
    let map = value.as_map().ok_or(string_t::from("Expected CBOR map for RPC request"))?;
    
    let mut method = String::new();
    let mut params = surrealdb::types::Array::new();
    
    for (k, v) in map {
        if let Some(key) = k.as_text() {
            match key {
                "method" => {
                    method = v.as_text().unwrap_or("").to_string();
                }
                "params" => {
                    if let Some(arr) = v.as_array() {
                        for item in arr {
                            params.push(cbor_to_value(item));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok((method, params))
}

fn cbor_to_value(v: &ciborium::Value) -> sdbValue {
    match v {
        ciborium::Value::Null => sdbValue::Null,
        ciborium::Value::Bool(b) => sdbValue::Bool(*b),
        ciborium::Value::Integer(i) => {
            let n: i128 = (*i).into();
            sdbValue::Number(surrealdb::types::Number::Int(n as i64))
        }
        ciborium::Value::Float(f) => sdbValue::Number(surrealdb::types::Number::Float(*f)),
        ciborium::Value::Text(s) => sdbValue::String(s.clone()),
        ciborium::Value::Bytes(b) => sdbValue::Bytes(surrealdb::types::Bytes::from(b.clone())),
        ciborium::Value::Array(arr) => {
            let vals: Vec<sdbValue> = arr.iter().map(cbor_to_value).collect();
            sdbValue::Array(surrealdb::types::Array::from(vals))
        }
        ciborium::Value::Map(map) => {
            let mut obj = surrealdb::types::Object::new();
            for (k, v) in map {
                if let Some(key) = k.as_text() {
                    obj.insert(key.to_string(), cbor_to_value(v));
                }
            }
            sdbValue::Object(obj)
        }
        _ => sdbValue::None,
    }
}

pub(crate) fn value_to_cbor(v: &sdbValue) -> ciborium::Value {
    match v {
        sdbValue::None => ciborium::Value::Null,
        sdbValue::Null => ciborium::Value::Null,
        sdbValue::Bool(b) => ciborium::Value::Bool(*b),
        sdbValue::Number(n) => match n {
            surrealdb::types::Number::Int(i) => ciborium::Value::Integer((*i).into()),
            surrealdb::types::Number::Float(f) => ciborium::Value::Float(*f),
            surrealdb::types::Number::Decimal(d) => ciborium::Value::Text(d.to_string()),
        },
        sdbValue::String(s) => ciborium::Value::Text(s.clone()),
        sdbValue::Array(arr) => {
            ciborium::Value::Array(arr.iter().map(value_to_cbor).collect())
        }
        sdbValue::Object(obj) => {
            let entries: Vec<(ciborium::Value, ciborium::Value)> = obj.iter()
                .map(|(k, v)| (ciborium::Value::Text(k.clone()), value_to_cbor(v)))
                .collect();
            ciborium::Value::Map(entries)
        }
        _ => ciborium::Value::Text(format!("{v:?}")),
    }
}

fn with_async<'a, 'b, C, F>(ctx: &'a SurrealRpc, err_ptr: *mut string_t, fun: C) -> c_int
where
    'a: 'b,
    C: FnOnce(&'a SurrealRpc) -> F + 'b,
    F: std::future::Future<Output = Result<c_int, string_t>>,
{
    // Same three corrections as the non-RPC path in lib.rs: report a poisoned
    // handle instead of aborting the host process, actually raise the poison
    // flag, and route every error through write_error so a null err_ptr --
    // which the header documents as legal -- does not segfault.
    if ctx.ps.load(Ordering::Acquire) {
        write_error(err_ptr, "rpc context is poisoned: an earlier operation returned SR_FATAL");
        return SR_FATAL;
    }
    let _guard = ctx.rt.enter();

    let res = match catch_unwind(AssertUnwindSafe(|| ctx.rt.block_on(fun(&ctx)))) {
        Ok(r) => r,
        Err(e) => {
            if let Some(e_str) = e.downcast_ref::<&str>() {
                write_error(err_ptr, format!("Panicked with: {e_str}"));
            } else {
                write_error(err_ptr, "Panicked");
            }
            ctx.ps.store(true, Ordering::Release);
            return SR_FATAL;
        }
    };

    match res {
        Ok(n) => n,
        Err(e) => {
            write_error(err_ptr, e);
            SR_ERROR
        }
    }
}

/// Decode a CBOR request, run it against `session`, and encode the reply.
///
/// Shared by `sr_surreal_rpc_execute` (which passes the context's own session)
/// and `sr_rpc_execute_on` (which passes a caller-chosen one). Since 3.1 the
/// session is a required argument -- requests cannot run unbound -- so the only
/// difference between the two entry points is where that uuid comes from.
async fn run_rpc(
    inner: &SurrealRpcInner,
    session: uuid::Uuid,
    res_ptr: *mut *mut u8,
    ptr: *const u8,
    len: c_int,
) -> Result<c_int, string_t> {
    let in_bytes = slice_from_raw_parts(ptr, len as usize);
    let in_bytes = unsafe { &*in_bytes };

    let in_value: ciborium::Value = ciborium::from_reader(in_bytes.as_ref())
        .map_err(|e| string_t::from(format!("CBOR decode error: {e}")))?;

    let (method_str, params) = parse_cbor_request(&in_value)?;
    let method = Method::parse_case_insensitive(&method_str);

    // (txn, session, client_session, method, params)
    //
    // `client_session` is not a duplicate of `session`: it marks the request as
    // belonging to a *named* session, and the trait persists only those -- a
    // per-request ephemeral is deliberately never written to durable storage.
    // Every session reachable from C is named (the caller's, or the one this
    // context minted for itself and keeps for its lifetime), so it is passed
    // whenever persistence is on. With no session directory the whole branch
    // is inert and this costs nothing.
    let client_session = inner.persist_sessions_enabled().then_some(session);

    let res = <SurrealRpcInner as RpcProtocol>::execute(
        inner,
        None,
        session,
        client_session,
        method,
        params,
    )
    .await
    .map_err(|e| string_t::from(e.to_string()))?;

    // Every DbResult variant is encoded, not just `Other`. `query` and `gql`
    // return `DbResult::Query`, so matching on `Other` alone made them
    // unreachable over RPC. `into_value` is core's own canonical mapping -- the
    // one the WebSocket and HTTP servers use -- so a C caller sees the same
    // shape as any other SurrealDB client: a query yields an array with one
    // entry per statement.
    let v = res.into_value();
    let cbor_val = value_to_cbor(&v);
    let mut out_bytes = Vec::new();
    ciborium::into_writer(&cbor_val, &mut out_bytes)
        .map_err(|e| string_t::from(format!("CBOR encode error: {e}")))?;
    let out = out_bytes.make_array();
    unsafe { res_ptr.write(out.ptr) }
    Ok(out.len)
}

/// Free the uuid array returned by `sr_rpc_session_list`.
#[export_name = "sr_uuid_arr_free"]
pub extern "C" fn uuid_arr_free(ptr: *mut Uuid, len: c_int) {
    ArrayGen { ptr, len }.free()
}

/// Write a file readable only by the current user.
///
/// `std::fs::write` would leave it at 0644 after a default umask, i.e. readable
/// by every account on the box -- and these files contain session tokens. The
/// mode is applied at creation and then again explicitly, because `mode()` has
/// no effect on a temporary file that already exists from an earlier crash.
#[cfg(unix)]
fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(bytes)
}

/// No mode bits to set; the file inherits the directory's ACL.
#[cfg(not(unix))]
fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)
}

/// Keep the session directory itself owner-only. Best effort: a caller may have
/// pointed us at a directory it does not own, and failing the write over that
/// would be worse than leaving the permissions as the operator set them.
#[cfg(unix)]
fn restrict_dir(dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn restrict_dir(_dir: &std::path::Path) {}

/// Who registered a live query, and where.
///
/// The namespace and database are recorded because cancelling a live query
/// means running `KILL` against the database it was registered in, and by the
/// time we need to do that the owning session may already be gone.
#[derive(Clone)]
struct LiveQueryOwner {
    session: uuid::Uuid,
    namespace: Option<String>,
    database: Option<String>,
}

#[allow(dead_code)]
struct SurrealRpcInner {
    kvs: Arc<Datastore>,
    /// Where attached sessions are persisted, or `None` when persistence is
    /// off. See the RpcProtocol impl below.
    session_dir: Option<std::path::PathBuf>,
    session_map: HashMap<uuid::Uuid, Arc<RwLock<Session>>>,
    /// Live queries registered through this context, keyed by live-query id.
    ///
    /// Upstream keeps no such registry: it hands us `handle_live` /
    /// `handle_kill` and expects the transport to track ownership, because only
    /// the transport knows which client a live query belongs to.
    live_queries: HashMap<uuid::Uuid, LiveQueryOwner>,
    /// Receiving half of the live-query notification channel handed to the
    /// datastore at construction. Cloned out by `sr_surreal_rpc_notifications`.
    notify_rx: Receiver<PublicNotification>,
    /// Id of the session every `sr_surreal_rpc_execute` call runs under.
    default_session: uuid::Uuid,
}

impl RpcProtocol for SurrealRpcInner {
    fn kvs(&self) -> &Datastore {
        self.kvs.as_ref()
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

    // ----------------------------------------------------------------------
    // Live query ownership
    // ----------------------------------------------------------------------
    //
    // These four hooks are the transport's half of live-query lifecycle.
    // Upstream deliberately has no default for `cleanup_lqs` /
    // `cleanup_all_lqs`, and defaults `handle_live` / `handle_kill` to
    // `unimplemented!()` when LQ_SUPPORT is true, because a live query outlives
    // the statement that created it and only the transport knows whose it is.
    //
    // Core calls `cleanup_lqs` from seven places -- `signup`, `signin`,
    // `authenticate`, `refresh`, `invalidate`, `reset` and `del_session` -- and
    // five of those are authentication changes rather than teardown. That is
    // the point: a live query registered under one identity must stop when the
    // session's identity changes, or it keeps streaming rows to a caller that
    // is no longer entitled to them.

    const LQ_SUPPORT: bool = true;

    async fn handle_live(
        &self,
        lqid: &uuid::Uuid,
        session_id: uuid::Uuid,
        namespace: Option<String>,
        database: Option<String>,
    ) {
        self.live_queries.insert(
            *lqid,
            LiveQueryOwner {
                session: session_id,
                namespace,
                database,
            },
        );
    }

    async fn handle_kill(&self, lqid: &uuid::Uuid) {
        // The KILL has already run; this only retires the bookkeeping.
        self.live_queries.remove(lqid);
    }

    async fn cleanup_lqs(&self, session_id: &uuid::Uuid) {
        let victims: Vec<(uuid::Uuid, LiveQueryOwner)> = self
            .live_queries
            .to_vec()
            .into_iter()
            .filter(|(_, owner)| &owner.session == session_id)
            .collect();
        self.kill_live_queries(victims).await;
    }

    async fn cleanup_all_lqs(&self) {
        // Core never calls this; it exists for a transport shutting down while
        // its datastore lives on. Ours usually goes away with the context, but
        // a caller that shares a datastore needs it to mean something.
        let victims = self.live_queries.to_vec();
        self.kill_live_queries(victims).await;
    }

    // ----------------------------------------------------------------------
    // Session persistence
    // ----------------------------------------------------------------------
    //
    // New in SurrealDB 3.2. Upstream added it for edge deployments, where an
    // idle isolate is evicted between requests and an attached session has to
    // be rehydrated on the next one. An embedded library has no such
    // lifecycle, but the same machinery lets a session outlive the RPC context
    // that created it -- reopen the context against the same directory and an
    // attached session is still there.
    //
    // Sessions are stored as one JSON file per id. The const is true so the
    // durability branches compile in; `persist_sessions_enabled` is the
    // runtime switch, and returns false unless the caller supplied a
    // directory, which is exactly the "operator configuration" shape the trait
    // documents.

    const PERSIST_SESSIONS: bool = true;

    fn persist_sessions_enabled(&self) -> bool {
        self.session_dir.is_some()
    }

    async fn load_session(&self, id: &uuid::Uuid) -> Option<Session> {
        let path = self.session_path(id)?;
        let bytes = std::fs::read(path).ok()?;
        // A file written by an incompatible version is treated as absent
        // rather than fatal: the session is simply not resumable.
        serde_json::from_slice(&bytes).ok()
    }

    async fn persist_session(&self, id: &uuid::Uuid, session: &Session) {
        let Some(path) = self.session_path(id) else {
            return;
        };
        // This is the whole session, which means the auth token, the record
        // auth data and the session variables, as plain JSON. See the warning
        // on `Options::session_dir`; the file mode below is the only thing
        // standing between that and any other account on the machine.
        let Ok(bytes) = serde_json::to_vec(session) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
            restrict_dir(dir);
        }
        // Written via a temporary file and renamed, so a crash mid-write
        // cannot leave a half-session that load_session would then reject.
        let tmp = path.with_extension("tmp");
        if write_private(&tmp, &bytes).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        } else {
            // Do not leave a partial file behind holding a token.
            let _ = std::fs::remove_file(&tmp);
        }
    }

    async fn forget_session(&self, id: &uuid::Uuid) -> Result<(), surrealdb::types::Error> {
        let Some(path) = self.session_path(id) else {
            return Ok(());
        };
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            // Already gone is success: del_session removes the durable copy
            // first and aborts on failure, so reporting an error here would
            // strand a session that is in fact already forgotten.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(surrealdb::types::Error::internal(format!(
                "could not forget session {id}: {e}"
            ))),
        }
    }
}

impl SurrealRpcInner {
    /// Cancel the given live queries and drop them from the registry.
    ///
    /// `KILL` is issued as the system rather than as the registering session:
    /// every caller of this arrives because that session was destroyed or
    /// re-authenticated, so its credentials are either gone or no longer the
    /// ones that should authorise the cancellation.
    ///
    /// The registry entry is removed first. A `KILL` that fails must not leave
    /// the id behind to be retried forever on every later cleanup, and a live
    /// query whose namespace was never recorded cannot be addressed by `KILL`
    /// at all.
    async fn kill_live_queries(&self, victims: Vec<(uuid::Uuid, LiveQueryOwner)>) {
        for (lqid, owner) in victims {
            self.live_queries.remove(&lqid);

            let (Some(ns), Some(db)) = (owner.namespace, owner.database) else {
                continue;
            };

            // `with_rt` is required, not decorative: KILL is a realtime
            // statement, and a session without it is refused with
            // LiveQueryNotSupported.
            let session = Session::owner().with_rt(true).with_ns(&ns).with_db(&db);
            let mut vars = surrealdb::types::Variables::new();
            vars.insert("lqid", sdbValue::Uuid(surrealdb::types::Uuid::from(lqid)));

            // Best effort: the live query may already be gone, and there is no
            // caller left to report a failure to.
            let _ = self.kvs.execute("KILL $lqid", &session, Some(vars)).await;
        }
    }

    /// Path of the file backing one session, or `None` when persistence is off.
    ///
    /// The uuid is rendered by `Uuid::to_string`, which is hyphenated hex and
    /// therefore always a safe file name -- a session id never reaches the
    /// filesystem as caller-controlled text.
    fn session_path(&self, id: &uuid::Uuid) -> Option<std::path::PathBuf> {
        let dir = self.session_dir.as_ref()?;
        Some(dir.join(format!("{id}.session.json")))
    }
}
