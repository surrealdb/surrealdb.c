pub mod types;
pub mod utils;

use std::{
    ffi::{c_char, c_int, CStr},
    future::IntoFuture,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::{AtomicBool, Ordering},
};

use stream::Stream;
use string::string_t;
use surrealdb::{
    engine::any::{self, Any},
    opt::Resource,
    sql, Surreal as sdbSurreal,
};
use tokio::runtime::Runtime;
use types::result::ArrayResult;

use array::{Array, ArrayGen, MakeArray};
pub use types::*;
use value::Value;

pub const SR_ERROR: c_int = -1;
pub const SR_FATAL: c_int = -2;
pub const SR_NONE: c_int = -3;

pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Runtime,
    ps: AtomicBool,
}

impl Surreal {
    #[export_name = "sr_connect"]
    pub extern "C" fn connect(
        err_ptr: *mut string_t,
        surreal_ptr: *mut *mut Surreal,
        endpoint: *const c_char,
    ) -> c_int {
        let res: Result<Surreal, string_t> = 'res: {
            let Ok(endpoint) = (unsafe { CStr::from_ptr(endpoint).to_str() }) else {
                break 'res Err("invalid utf8".into());
            };

            let Ok(rt) = Runtime::new() else {
                break 'res Err("error creating runtime".into());
            };

            let con_fut = any::connect(endpoint);

            let db = match rt.block_on(con_fut.into_future()) {
                Ok(db) => db,
                Err(e) => break 'res Err(e.into()),
            };

            Ok(Surreal {
                db,
                rt,
                ps: AtomicBool::new(false),
            })
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

    #[export_name = "sr_surreal_disconnect"]
    pub extern "C" fn disconnect(db: *mut Surreal) {
        let _ = unsafe { Box::from_raw(db) };
    }

    // authenticate.rs

    // begin.rs

    // cancel.rs

    // commit.rs

    // content.rs

    // create.rs

    // delete.rs

    // export.rs

    // health.rs

    // import.rs

    // insert.rs

    // invalidate.rs

    // live.rs
    /// if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
    /// this pointer should not be copied and only one should be used at a time
    #[export_name = "sr_select_live"]
    pub extern "C" fn select_live(
        db: &Surreal,
        err_ptr: *mut string_t,
        stream_ptr: *mut &mut Stream,
        resource: *const c_char,
    ) -> c_int {
        use surrealdb::method::Stream as sdbStream;
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let fut = surreal.db.select(Resource::from(resource)).live();

            let stream_inner: sdbStream<sql::Value> = surreal.rt.block_on(fut.into_future())?;

            let stream_boxed = Box::new(Stream::new(stream_inner, surreal.rt.handle().clone()));

            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };

            Ok(1)
        })
    }

    // merge.rs

    // mod.rs

    // patch.rs

    // query.rs
    #[export_name = "sr_query"]
    pub extern "C" fn query(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut ArrayResult,
        query: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let query = unsafe { CStr::from_ptr(query) }.to_str()?;

            let mut res = surreal.db.query(query).await?;
            let res_len = res.num_statements();

            let mut acc = Vec::with_capacity(res_len);
            for index in 0..res_len {
                let arr_res = match res.take::<sql::Value>(index) {
                    Ok(sql::Value::Array(arr)) => {
                        let a = arr.into();
                        ArrayResult::ok(a)
                    }
                    Ok(val) => {
                        let arr: Array = vec![val.into()].into();
                        ArrayResult::ok(arr)
                    }
                    Err(e) => ArrayResult::err(e.to_string()),
                };
                acc.push(arr_res);
            }

            let ArrayGen { ptr, len } = acc.make_array();
            unsafe { res_ptr.write(ptr) }

            Ok(len)
        })
    }

    // select.rs
    #[export_name = "sr_select"]
    pub extern "C" fn select(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let res = match surreal.db.select(Resource::from(resource)).await? {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            unsafe { res_ptr.write(res.arr) }

            Ok(res.len as c_int)
        })
    }
    // set.rs

    // signin.rs

    // signup.rs

    // unset.rs

    // update.rs

    // upsert.rs

    // use_db.rs
    #[export_name = "sr_use_db"]
    pub extern "C" fn use_db(db: &Surreal, err_ptr: *mut string_t, query: *const c_char) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let db_name = unsafe { CStr::from_ptr(query) }.to_str()?;

            surreal.db.use_db(db_name).await?;

            Ok(0)
        })
    }
    // use_ns.rs
    #[export_name = "sr_use_ns"]
    pub extern "C" fn use_ns(db: &Surreal, err_ptr: *mut string_t, query: *const c_char) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let ns_name = unsafe { CStr::from_ptr(query) }.to_str()?;

            surreal.db.use_db(ns_name).await?;

            Ok(0)
        })
    }

    // version.rs

    #[export_name = "sr_version"]
    pub extern "C" fn version(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut string_t,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            // let fut = surreal.db.version();

            // let res = surreal.rt.block_on(fut.into_future())?;
            let res = surreal.db.version().await?;
            let res_str: string_t = res.into();

            unsafe { res_ptr.write(res_str) }

            return Ok(0);
        })
    }
}

fn with_surreal_async<'a, 'b, C, F>(db: &'a Surreal, err_ptr: *mut string_t, fun: C) -> c_int
where
    'a: 'b,
    C: FnOnce(&'a Surreal) -> F + 'b,
    F: std::future::Future<Output = Result<c_int, string_t>>,
{
    if db.ps.load(Ordering::Acquire) {
        std::process::abort()
    }

    let res = match catch_unwind(AssertUnwindSafe(|| db.rt.block_on(fun(&db)))) {
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
