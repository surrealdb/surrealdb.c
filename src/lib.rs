pub mod result;
pub mod types;
use std::{
    ffi::{c_char, CStr},
    future::IntoFuture,
};

use futures::StreamExt;
use result::{ArrayResult, ArrayResultArray, ArrayResultArrayResult, StringResult, SurrealResult};
use stream::{Stream, StreamResult};
use surrealdb::{
    engine::any::{self, Any},
    opt::Resource,
    sql, Surreal as sdbSurreal,
};
use tokio::runtime::Runtime;

use array::Array;
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

    #[no_mangle]
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
    #[no_mangle]
    pub extern "C" fn select_live(db: *mut Surreal, resource: *const c_char) -> StreamResult {
        use surrealdb::method::Stream as sdbStream;
        with_surreal(db, |surreal| {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str().unwrap();
            let fut = surreal.db.select(Resource::from(resource)).live();

            let stream: sdbStream<Any, sql::Value> = match surreal.rt.block_on(fut.into_future()) {
                Ok(s) => s,
                Err(e) => return StreamResult::err(e),
            };

            let out = Box::new(Stream::new(stream, surreal.rt.handle().clone()));

            StreamResult::ok(Box::leak(out))
        })
    }

    // merge.rs

    // mod.rs

    // patch.rs

    // query.rs
    #[no_mangle]
    pub extern "C" fn query(db: *mut Surreal, query: *const c_char) -> ArrayResultArrayResult {
        with_surreal(db, |surreal| {
            let query = unsafe { CStr::from_ptr(query) }
                .to_str()
                .expect("Query should be valid utf-8");

            let fut = surreal.db.query(query);

            let mut res = match surreal.rt.block_on(fut.into_future()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{e}");
                    return ArrayResultArrayResult::err(e);
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
            ArrayResultArrayResult::ok(arr_res_arr)
        })
    }

    // select.rs
    #[no_mangle]
    pub extern "C" fn select(db: *mut Surreal, resource: *const c_char) -> ArrayResult {
        with_surreal(db, |surreal| {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str().unwrap();

            // let fut = surreal.db.select(resource);

            // let res: Vec<BTreeMap<String, sql::Value>> =
            //     surreal.rt.block_on(fut.into_future()).unwrap();

            let fut = surreal.db.select(Resource::from(resource));

            let res = match surreal.rt.block_on(fut.into_future()) {
                Ok(sql::Value::Array(a)) => ArrayResult::ok(Array::from(a)),
                Ok(v) => {
                    // let foo: Array = v;
                    ArrayResult::ok(Array::from(vec![v.into()]))
                }
                Err(e) => ArrayResult::err(e),
            };

            // CString::new(format!("{res:?}")).unwrap().into_raw()
            res
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
