pub mod types;
use std::{
    ffi::{self, c_char, CStr, CString},
    future::IntoFuture,
    mem::ManuallyDrop,
    ptr,
};

use surrealdb::{
    engine::any::{self, Any},
    Surreal as sdbSurreal,
};
use surrealdb_c_macro::with_surreal;
use tokio::runtime::Runtime;

pub use types::*;

pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Runtime,
}
#[no_mangle]
pub extern "C" fn connect(endpoint: *const c_char) -> *mut Surreal {
    let endpoint = unsafe { CStr::from_ptr(endpoint).to_str().unwrap() };

    let rt = Runtime::new().unwrap();

    let con_fut = any::connect(endpoint);

    let db = rt.block_on(con_fut.into_future()).unwrap();

    let boxed = Box::new(Surreal { db, rt });

    return Box::leak(boxed);
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
// select.rs
// set.rs
// signin.rs
// signup.rs
// tests
// unset.rs
// update.rs
// upsert.rs
// use_db.rs
// use_ns.rs

// version.rs
#[no_mangle]
pub extern "C" fn version(db: *mut Surreal) -> *mut c_char {
    let surreal = unsafe { Box::from_raw(db) };

    let fut = surreal.db.version();

    let res = surreal.rt.block_on(fut.into_future()).unwrap();
    let ver_str = CString::new(res.to_string()).unwrap().into_raw();

    Box::leak(surreal);
    return ver_str;
}

// fn with_surreal<C, F, O>(surreal: *mut Surreal, fun: C) -> O
// where
//     C: Fn(&sdbSurreal<Any>) -> F,
//     F: IntoFuture<Output = O>,
// {
// let surreal = unsafe { Box::from_raw(surreal) };

// let fut: F = fun(&surreal.db);
// let res = surreal.rt.block_on(fut.into_future());

// Box::leak(surreal);
// res
// }
