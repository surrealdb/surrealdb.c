pub mod result;
pub mod types;
use std::{
    collections::BTreeMap,
    ffi::{self, c_char, c_int, CStr, CString},
    fmt::format,
    future::IntoFuture,
    mem::ManuallyDrop,
    ptr,
};

use result::{StringResult, SurrealResult};
use surrealdb::{
    engine::any::{self, Any},
    method::Select,
    sql, Surreal as sdbSurreal,
};
use surrealdb_c_macro::def_result;
use tokio::runtime::Runtime;

pub use types::*;

pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Runtime,
}

impl Surreal {
    #[no_mangle]
    pub extern "C" fn connect(endpoint: *const c_char) -> SurrealResult {
        let endpoint = unsafe { CStr::from_ptr(endpoint).to_str().unwrap() };

        let rt = Runtime::new().unwrap();

        let con_fut = any::connect(endpoint);

        let db = match rt.block_on(con_fut.into_future()) {
            Ok(db) => db,
            Err(e) => return SurrealResult::err(e.to_string()),
        };

        let boxed = Box::new(Surreal { db, rt });

        return SurrealResult::ok(Box::leak(boxed));
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
    // merge.rs
    // mod.rs
    // patch.rs
    // query.rs
    #[no_mangle]
    pub extern "C" fn query(db: *mut Surreal, query: *const c_char) -> StringResult {
        with_surreal(db, |surreal| {
            let query = unsafe { CStr::from_ptr(query) }
                .to_str()
                .expect("Query should be valid utf-8");

            let fut = surreal.db.query(query);

            let res = match surreal.rt.block_on(fut.into_future()) {
                Ok(r) => r,
                Err(e) => return StringResult::err(e),
            };

            // CString::new(format!("{res:?}")).unwrap().into_raw()
            StringResult::ok(format!("{res:?}"))
        })
    }

    // select.rs
    #[no_mangle]
    pub extern "C" fn select(db: *mut Surreal, resource: *const c_char) -> *mut c_char {
        with_surreal(db, |surreal| {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str().unwrap();

            let fut = surreal.db.select(resource);

            let res: Vec<BTreeMap<String, sql::Value>> =
                surreal.rt.block_on(fut.into_future()).unwrap();

            CString::new(format!("{res:?}")).unwrap().into_raw()
        })
    }
    // set.rs
    // signin.rs
    // signup.rs
    // unset.rs
    // update.rs
    // upsert.rs

    // use_db.rs
    #[no_mangle]
    pub extern "C" fn use_db(db: *mut Surreal, query: *const c_char) {
        with_surreal(db, |surreal| {
            let db = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

            let fut = surreal.db.use_db(db);

            surreal.rt.block_on(fut.into_future()).unwrap();
        })
    }
    // use_ns.rs
    #[no_mangle]
    pub extern "C" fn use_ns(db: *mut Surreal, query: *const c_char) {
        with_surreal(db, |surreal| {
            let ns = unsafe { CStr::from_ptr(query) }.to_str().unwrap();

            let fut = surreal.db.use_ns(ns);

            surreal.rt.block_on(fut.into_future()).unwrap();
        })
    }

    // version.rs
    // #[no_mangle]
    // pub extern "C" fn version(db: *mut Surreal) -> *mut c_char {
    //     let surreal = unsafe { Box::from_raw(db) };

    //     let fut = surreal.db.version();

    //     let res = surreal.rt.block_on(fut.into_future()).unwrap();
    //     let ver_str = CString::new(res.to_string()).unwrap().into_raw();

    //     Box::leak(surreal);
    //     return ver_str;
    // }

    #[no_mangle]
    pub extern "C" fn version(db: *mut Surreal) -> StringResult {
        with_surreal(db, |surreal| {
            let fut = surreal.db.version();

            let res = match surreal.rt.block_on(fut.into_future()) {
                Ok(r) => r,
                Err(e) => return StringResult::err(e),
            };

            return StringResult::ok(res.to_string());
        })
    }
}

fn with_surreal<C, O>(db: *mut Surreal, fun: C) -> O
where
    C: Fn(&Surreal) -> O,
{
    let surreal = unsafe { Box::from_raw(db) };

    let res = fun(&surreal);

    Box::leak(surreal);
    res
}
