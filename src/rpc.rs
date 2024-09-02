use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, CStr},
    future::IntoFuture,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::slice_from_raw_parts,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use surrealdb::dbs::Session;
use surrealdb::kvs::Datastore;
use surrealdb::rpc::format::cbor;
use surrealdb::rpc::method::Method;
use surrealdb::rpc::rpc_context::RpcContext;
use surrealdb::rpc::Data;
use surrealdb::sql;
use tokio::{runtime::Runtime, sync::RwLock};

use crate::{
    array::MakeArray, opts::Options, stream::RpcStream, string::string_t, SR_ERROR, SR_FATAL,
};

/// The object representing a Surreal connection
///
/// It is safe to be referenced from multiple threads
/// If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
/// (use will cause the program to abort)
///
/// should be freed with sr_surreal_disconnect
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
        // TODO: live query support
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
                Err(e) => return Err(e.into()),
            };

            kvs = kvs.with_notifications();

            kvs = kvs.with_strict_mode(options.strict);
            if options.query_timeout != 0 {
                kvs =
                    kvs.with_query_timeout(Some(Duration::from_secs(options.query_timeout as u64)))
            }
            if options.transaction_timeout != 0 {
                kvs = kvs.with_transaction_timeout(Some(Duration::from_secs(
                    options.transaction_timeout as u64,
                )))
            }

            let inner = SurrealRpcInner {
                kvs,
                sess: Session::default().with_rt(true),
                vars: BTreeMap::default(),
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

    /// execute rpc
    ///
    /// free result with sr_free_byte_arr
    #[export_name = "sr_surreal_rpc_execute"]
    pub extern "C" fn execute(
        &self,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut u8,
        ptr: *const u8,
        len: c_int,
    ) -> c_int {
        with_async(self, err_ptr, |ctx| async {
            let in_bytes = slice_from_raw_parts(ptr, len as usize);
            let in_bytes = unsafe { &*in_bytes };
            let in_data = cbor::req(in_bytes.to_vec())?;
            let method = Method::parse(in_data.method);
            let res = match method.can_be_immut() {
                true => {
                    ctx.inner
                        .read()
                        .await
                        .execute_immut(method, in_data.params)
                        .await
                }
                false => {
                    ctx.inner
                        .write()
                        .await
                        .execute(method, in_data.params)
                        .await
                }
            }?;
            let out = cbor::res(res)?.make_array();
            unsafe { res_ptr.write(out.ptr) }
            Ok(out.len)
        })
    }

    #[export_name = "sr_surreal_rpc_notifications"]
    pub extern "C" fn notifications(
        &self,
        err_ptr: *mut string_t,
        stream_ptr: *mut &mut RpcStream,
    ) -> c_int {
        with_async(self, err_ptr, |ctx| async {
            let stream = ctx
                .inner
                .read()
                .await
                .kvs
                .notifications()
                .ok_or("Notifications not enabled")?;

            Ok(0)
        })
    }

    #[export_name = "sr_surreal_rpc_free"]
    pub extern "C" fn rpc_free(ctx: *mut SurrealRpc) {
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

struct SurrealRpcInner {
    kvs: Datastore,
    sess: Session,
    vars: BTreeMap<String, sql::Value>,
}

impl RpcContext for SurrealRpcInner {
    fn kvs(&self) -> &Datastore {
        &self.kvs
    }

    fn session(&self) -> &Session {
        &self.sess
    }

    fn session_mut(&mut self) -> &mut Session {
        &mut self.sess
    }

    fn vars(&self) -> &std::collections::BTreeMap<String, sql::Value> {
        &self.vars
    }

    fn vars_mut(&mut self) -> &mut std::collections::BTreeMap<String, sql::Value> {
        &mut self.vars
    }

    fn version_data(&self) -> Data {
        let ver_str = surrealdb_core::env::VERSION.to_string();
        ver_str.into()
    }

    const LQ_SUPPORT: bool = true;
    async fn handle_live(&self, _lqid: &uuid::Uuid) {}
    async fn handle_kill(&self, _lqid: &uuid::Uuid) {}
}
