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
use value::{Object, Value};

pub const SR_ERROR: c_int = -1;
pub const SR_FATAL: c_int = -2;
pub const SR_NONE: c_int = -3;

pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Runtime,
    ps: AtomicBool,
}

impl Surreal {
    /// connects to a local, remote, or embedded database
    ///
    /// if any function returns SR_FATAL, this must not be used (except to drop) (TODO: check this is safe) doing so will cause the program to abort
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_string_t err;
    /// sr_surreal_t *db;
    ///
    /// // connect to in-memory instance
    /// if (sr_connect(&err, &db, "mem://") < 0) {
    ///     printf("error connecting to db: %s\n", err);
    ///     return 1;
    /// }
    ///
    /// // connect to surrealkv file
    /// if (sr_connect(&err, &db, "surrealkv://test.skv") < 0) {
    ///     printf("error connecting to db: %s\n", err);
    ///     return 1;
    /// }
    ///
    /// // connect to surrealdb server
    /// if (sr_connect(&err, &db, "wss://localhost:8000") < 0) {
    ///     printf("error connecting to db: %s\n", err);
    ///     return 1;
    /// }
    ///
    /// sr_surreal_disconnect(db);
    /// ```
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

    /// disconnect a database connection
    /// note: the Surreal object must not be used after this function has been called
    ///     any object allocations will still be valid, and should be freed, using the appropriate function
    /// TODO: check if Stream can be freed after disconnection because of rt
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// // connect
    /// disconnect(db);
    /// ```
    #[export_name = "sr_surreal_disconnect"]
    pub extern "C" fn disconnect(db: *mut Surreal) {
        // TODO(raphaeldarley): catch panics
        let _ = unsafe { Box::from_raw(db) };
    }

    // authenticate.rs

    // begin.rs

    // cancel.rs

    // commit.rs

    // content.rs

    // create.rs

    // #[export_name = "sr_create"]
    // pub extern "C" fn create(
    //     db: &Surreal,
    //     err_ptr: *mut string_t,
    //     res_ptr: *mut ,
    //     resource: *const c_char,
    // ) -> c_int {
    //     with_surreal_async(db, err_ptr, |surreal| async {
    //         let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
    //     })
    // }

    // delete.rs

    // export.rs

    // health.rs

    // import.rs

    // insert.rs

    // invalidate.rs

    // live.rs
    /// make a live selection
    /// if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
    /// which can be moved accross threads but not aliased
    ///
    /// # Examples
    ///
    /// sr_stream_t *stream;
    /// if (sr_select_live(db, &err, &stream, "foo") < 0)
    /// {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    ///
    /// sr_notification_t not ;
    /// if (sr_stream_next(stream, &not ) > 0)
    /// {
    ///     sr_print_notification(&not );
    /// }
    /// sr_stream_kill(stream);
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
            println!("got resource: {resource}");

            let stream_inner: sdbStream<sql::Value> =
                surreal.db.select(Resource::from(resource)).live().await?;
            println!("got stream: {stream_inner:?}");

            let stream_boxed = Box::new(Stream::new(stream_inner, surreal.rt.handle().clone()));

            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };
            println!("written back stream");

            Ok(1)
        })
    }

    // merge.rs

    // mod.rs

    // patch.rs

    // query.rs
    // #[export_name = "sr_query"]
    // pub extern "C" fn query(
    //     db: &Surreal,
    //     err_ptr: *mut string_t,
    //     res_ptr: *mut *mut ArrayResult,
    //     query: *const c_char,
    //     vars: *const Object,
    // ) -> c_int {
    //     with_surreal_async(db, err_ptr, |surreal| async {
    //         let query = unsafe { CStr::from_ptr(query) }.to_str()?;
    //         let vars: sql::Object = match vars.is_null() {
    //             true => Default::default(),
    //             false => unsafe { &*vars }.clone().into(),
    //         };

    //         println!("got vars: {vars:?}");

    //         // let mut res = surreal.db.query(query).bind(vars).await?;
    //         let mut res = surreal.db.query(query).await?;
    //         println!("got res: {res:?}");
    //         let res_len = res.num_statements();

    //         let mut acc = Vec::with_capacity(res_len);
    //         for index in 0..res_len {
    //             let arr_res = match res.take::<sql::Value>(index) {
    //                 Ok(sql::Value::Array(arr)) => {
    //                     let a = arr.into();
    //                     ArrayResult::ok(a)
    //                 }
    //                 Ok(val) => {
    //                     let arr: Array = vec![val.into()].into();
    //                     ArrayResult::ok(arr)
    //                 }
    //                 Err(e) => ArrayResult::err(e.to_string()),
    //             };
    //             acc.push(arr_res);
    //         }

    //         let ArrayGen { ptr, len } = acc.make_array();
    //         unsafe { res_ptr.write(ptr) }

    //         Ok(len)
    //     })
    // }
    #[export_name = "sr_query"]
    pub extern "C" fn query(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut ArrayResult,
        query: *const c_char,
        vars: *const Object,
    ) -> c_int {
        with_surreal(db, err_ptr, |surreal| {
            let query = unsafe { CStr::from_ptr(query) }.to_str()?;
            let vars: sql::Object = match vars.is_null() {
                true => Default::default(),
                false => unsafe { &*vars }.clone().into(),
            };

            println!("got vars: {vars:?}");

            // let mut res = surreal.db.query(query).bind(vars).await?;
            let fut = surreal.db.query(query).into_future();
            // let mut res = surreal.db.query(query).await?;
            let mut res = surreal.rt.block_on(fut)?;
            println!("got res: {res:?}");
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
    /// query the database
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
        let out = with_surreal_async(db, err_ptr, |surreal| async {
            let ns_name = unsafe { CStr::from_ptr(query) }.to_str()?;

            surreal.db.use_ns(ns_name).await?;

            Ok(0)
        });
        out
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

fn with_surreal<C>(db: &Surreal, err_ptr: *mut string_t, fun: C) -> c_int
where
    C: FnOnce(&Surreal) -> Result<c_int, string_t>,
{
    if db.ps.load(Ordering::Acquire) {
        std::process::abort()
    }

    let res = match catch_unwind(AssertUnwindSafe(|| fun(&db))) {
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
