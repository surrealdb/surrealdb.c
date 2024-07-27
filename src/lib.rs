pub mod types;
pub mod utils;

use std::{
    ffi::{c_char, CStr},
    future::IntoFuture,
    mem, ptr,
    sync::Arc,
};

use result::SurrealResult;
use stream::Stream;
use string::string_t;
use surrealdb::{
    engine::any::{self, Any},
    opt::Resource,
    sql, Surreal as sdbSurreal,
};
use tokio::runtime::Runtime;
use types::result::{ArrayResult, ArrayResultArray};

use array::Array;
pub use types::*;
use utils::Empty;

pub struct SurrealInner {
    db: sdbSurreal<Any>,
    rt: Runtime,
}

/// a type representing a shared SurrealDB connection and local errors
/// pointers can be shared across threads,
/// but copy must be called before any other functions are used to prevent race conditions
#[repr(C)]
pub struct Surreal {
    inner: *const SurrealInner,
    err: string_t,
}

impl Surreal {
    pub fn null() -> Surreal {
        Surreal {
            inner: ptr::null(),
            err: string_t::null(),
        }
    }
}

impl Surreal {
    #[export_name = "sr_connect"]
    pub extern "C" fn connect(endpoint: *const c_char) -> SurrealResult {
        let endpoint = unsafe { CStr::from_ptr(endpoint).to_str().unwrap() };

        let rt = Runtime::new().unwrap();

        let con_fut = any::connect(endpoint);

        let db = match rt.block_on(con_fut.into_future()) {
            Ok(db) => db,
            Err(e) => return SurrealResult::err(e.to_string()),
        };

        let boxed = Arc::new(SurrealInner { db, rt });
        let inner = Arc::into_raw(boxed);

        let out = Surreal {
            inner,
            err: string_t::null(),
        };

        return SurrealResult::ok(out);
    }

    /// shallow copies surrealdb connection that can be passed between threads and has seperate error handling
    #[export_name = "sr_surreal_copy"]
    pub extern "C" fn copy(surreal: &Surreal) -> Surreal {
        // turn pointer into Arc to clone, then turn it back to maintain count
        let old = surreal.inner;
        let arced_old = unsafe { Arc::from_raw(old) };
        #[cfg(debug_assertions)]
        let old_count = Arc::strong_count(&arced_old);

        let arced_new = arced_old.clone();

        #[cfg(debug_assertions)]
        let new_count = Arc::strong_count(&arced_new);
        debug_assert_eq!(old_count + 1, new_count);

        let _ = Arc::into_raw(arced_old);
        Surreal {
            inner: Arc::into_raw(arced_new),
            err: string_t::null(),
        }
    }

    #[export_name = "sr_surreal_disconnect"]
    pub extern "C" fn disconnect(db: Surreal) {
        let _ = db;
    }

    /// takes error from a Sureal connection, leaving it blank
    /// useful when errors are expected or recoverable
    #[export_name = "sr_err"]
    pub extern "C" fn get_err(db: &mut Surreal) -> string_t {
        mem::take(&mut db.err)
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
    #[export_name = "sr_select_live"]
    pub extern "C" fn select_live(db: &mut Surreal, resource: *const c_char) -> *mut Stream {
        use surrealdb::method::Stream as sdbStream;
        with_surreal(db, |surreal| {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str().unwrap();
            let fut = surreal.db.select(Resource::from(resource)).live();

            // let stream: sdbStream<sql::Value> = match surreal.rt.block_on(fut.into_future()) {
            //     Ok(s) => s,
            //     Err(e) => {
            //         *err = e.to_string().to_string_t();
            //         return ptr::null_mut();
            //     }
            // };

            let stream: sdbStream<sql::Value> = surreal.rt.block_on(fut.into_future())?;

            let out = Box::new(Stream::new(stream, surreal.rt.handle().clone()));

            Ok(Box::leak(out) as *mut Stream)
        })
    }

    // merge.rs

    // mod.rs

    // patch.rs

    // query.rs
    #[export_name = "sr_query"]
    pub extern "C" fn query(db: &mut Surreal, query: *const c_char) -> ArrayResultArray {
        with_surreal(db, |surreal| {
            let query = unsafe { CStr::from_ptr(query) }
                .to_str()
                .expect("Query should be valid utf-8");

            let fut = surreal.db.query(query);

            let mut res = match surreal.rt.block_on(fut.into_future()) {
                Ok(r) => r,
                Err(e) => {
                    return Err(e.into());
                }
            };
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
            let arr_res_arr: ArrayResultArray = acc.into();
            Ok(arr_res_arr)
        })
    }

    // select.rs
    #[export_name = "sr_select"]
    pub extern "C" fn select(db: &mut Surreal, resource: *const c_char) -> Array {
        with_surreal(db, |surreal| {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str().unwrap();

            // let fut = surreal.db.select(resource);

            // let res: Vec<BTreeMap<String, sql::Value>> =
            //     surreal.rt.block_on(fut.into_future()).unwrap();

            let fut = surreal.db.select(Resource::from(resource));

            let res = match surreal.rt.block_on(fut.into_future()) {
                Ok(sql::Value::Array(a)) => Array::from(a),
                Ok(v) => {
                    // let foo: Array = v;
                    Array::from(vec![v.into()])
                }
                Err(e) => {
                    return Err(e.into());
                }
            };

            Ok(res)
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
    pub extern "C" fn use_db(db: &mut Surreal, query: *const c_char) {
        with_surreal(db, |surreal| {
            let db = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

            let fut = surreal.db.use_db(db);

            surreal.rt.block_on(fut.into_future()).unwrap();
            Ok(())
        })
    }
    // use_ns.rs
    #[export_name = "sr_use_ns"]
    pub extern "C" fn use_ns(db: &mut Surreal, query: *const c_char) {
        with_surreal(db, |surreal| {
            let ns = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

            let fut = surreal.db.use_ns(ns);

            surreal.rt.block_on(fut.into_future()).unwrap();
            Ok(())
        })
    }

    // version.rs

    #[export_name = "sr_version"]
    pub extern "C" fn version(db: &mut Surreal) -> string_t {
        with_surreal(db, |surreal| {
            let fut = surreal.db.version();

            // let res = match surreal.rt.block_on(fut.into_future()) {
            //     Ok(r) => r,
            //     Err(e) => return StringResult::err(e),
            // };
            let res = surreal.rt.block_on(fut.into_future())?;

            return Ok(res.into());
        })
    }
}

fn with_surreal<C, O>(db: &mut Surreal, fun: C) -> O
where
    C: Fn(&SurrealInner) -> Result<O, string_t>,
    O: Empty,
    // E: std::error::Error,
{
    let inner_arc = unsafe { Arc::from_raw(db.inner) };
    let inner = inner_arc.as_ref();

    let res = fun(&inner);

    let _ = Arc::into_raw(inner_arc);

    match res {
        Ok(o) => o,
        Err(e) => {
            db.err = e;
            O::empty()
        }
    }
}

// fn with_surreal_err<C, O>(db: &mut Surreal, fun: C) -> O
// where
//     C: Fn(&SurrealInner, &mut string_t) -> O,
// {
//     let inner_arc = unsafe { Arc::from_raw(db.inner) };
//     let inner = inner_arc.as_ref();

//     let res = fun(&inner, &mut db.err);

//     let _ = Arc::into_raw(inner_arc);

//     res
// }

impl Drop for Surreal {
    fn drop(&mut self) {
        let _arced_inner = unsafe { Arc::from_raw(self.inner) };
        let _err = string_t(self.err.0);
    }
}
