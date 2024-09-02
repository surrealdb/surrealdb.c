pub mod opts;
pub mod rpc;
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
    sql, Surreal as sdbSurreal, Value as apiValue,
};
use tokio::runtime::Runtime;
use types::result::ArrayResult;

use array::{Array, ArrayGen, MakeArray};
pub use types::*;
use utils::CStringExt2;
use value::{Object, Value};

pub const SR_NONE: c_int = 0;
pub const SR_CLOSED: c_int = -1;
pub const SR_ERROR: c_int = -2;
pub const SR_FATAL: c_int = -3;

/// The object representing a Surreal connection
///
/// It is safe to be referenced from multiple threads
/// If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
/// (use will cause the program to abort)
///
/// should be freed with sr_surreal_disconnect
pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Runtime,
    ps: AtomicBool,
}

// struct SurrealInner {
//     kvs: Datastore,
//     sess: Session,
// }

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
        // TODO: wrap in catch unwind
        let res: Result<Result<Surreal, string_t>, _> = catch_unwind(AssertUnwindSafe(|| {
            let Ok(endpoint) = (unsafe { CStr::from_ptr(endpoint).to_str() }) else {
                return Err("invalid utf8".into());
            };

            let Ok(rt) = Runtime::new() else {
                return Err("error creating runtime".into());
            };

            let con_fut = any::connect(endpoint);

            let db = match rt.block_on(con_fut.into_future()) {
                Ok(db) => db,
                Err(e) => return Err(e.into()),
            };

            Ok(Surreal {
                db,
                rt,
                ps: AtomicBool::new(false),
            })
        }));

        let res: Result<Surreal, string_t> = match res {
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
        catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(db) }))).ok();
    }

    // authenticate.rs

    // begin.rs

    // cancel.rs

    // commit.rs

    // content.rs

    // create.rs

    /// create a record
    ///
    #[export_name = "sr_create"]
    pub extern "C" fn create(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut &mut Object,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = surreal
                .db
                .create(Resource::from(resource))
                .content(content)
                .await?;
            let obj = match res.into_inner() {
                sql::Value::Object(o) => o,
                other => {
                    return Err(format!(
                        "Expected object as return type of create, but found: {other:?}"
                    )
                    .into())
                }
            };
            if !res_ptr.is_null() {
                let boxed = Box::new(obj.into());
                unsafe { res_ptr.write(Box::leak(boxed)) }
                Ok(1)
            } else {
                Ok(0)
            }
        })
    }

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

            let stream_inner: sdbStream<apiValue> =
                surreal.db.select(Resource::from(resource)).live().await?;

            let stream_boxed = Box::new(Stream::new(stream_inner, surreal.rt.handle().clone()));

            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };

            // return 1 because 1 stream was written to *stream_ptr
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
        vars: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let query = unsafe { CStr::from_ptr(query) }.to_str()?;
            let vars: sql::Object = match vars.is_null() {
                true => Default::default(),
                false => unsafe { &*vars }.clone().into(),
            };

            let mut res = surreal.db.query(query).bind(vars).await?;
            let res_len = res.num_statements();

            let mut acc = Vec::with_capacity(res_len);
            for index in 0..res_len {
                let api_res = res.take::<apiValue>(index);
                let arr_res = match api_res.map(|v| v.into_inner()) {
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
    /// select a resource
    ///
    /// can be used to select everything from a table or a single record
    /// writes values to *res_ptr, and returns number of values
    /// result values are allocated by Surreal and must be freed with sr_free_arr
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *foos;
    /// int len = sr_select(db, &err, &foos, "foo");
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    /// for (int i = 0; i < len; i++)
    /// {
    ///     sr_value_print(&foos[i]);
    /// }
    /// sr_free_arr(foos, len);
    #[export_name = "sr_select"]
    pub extern "C" fn select(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let res = match surreal
                .db
                .select(Resource::from(resource))
                .await?
                .into_inner()
            {
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
    /// select database
    /// NOTE: namespace must be selected first with sr_use_ns
    ///
    /// # Examples
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_use_db(db, &err, "test") < 0)
    /// {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_use_db"]
    pub extern "C" fn use_db(db: &Surreal, err_ptr: *mut string_t, query: *const c_char) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let db_name = unsafe { CStr::from_ptr(query) }.to_str()?;

            surreal.db.use_db(db_name).await?;

            Ok(0)
        })
    }
    // use_ns.rs
    /// select namespace
    /// NOTE: database must be selected before use with sr_use_db
    ///
    /// # Examples
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_use_ns(db, &err, "test") < 0)
    /// {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
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

    /// returns the db version
    /// NOTE: version is allocated in Surreal and must be freed with sr_free_string
    /// # Examples
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_string_t ver;
    ///
    /// if (sr_version(db, &err, &ver) < 0)
    /// {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// printf("%s", ver);
    /// sr_free_string(ver);
    /// ```
    #[export_name = "sr_version"]
    pub extern "C" fn version(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut string_t,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let res = surreal.db.version().await?;
            let res_string = res.to_string();
            let len = res_string.bytes().len();
            let res_str: string_t = res_string.to_string_t();

            unsafe { res_ptr.write(res_str) }

            return Ok(len as c_int);
        })
    }
}

// fn with_surreal<C>(db: &Surreal, err_ptr: *mut string_t, fun: C) -> c_int
// where
//     C: FnOnce(&Surreal) -> Result<c_int, string_t>,
// {
//     if db.ps.load(Ordering::Acquire) {
//         std::process::abort()
//     }

//     let res = match catch_unwind(AssertUnwindSafe(|| fun(&db))) {
//         Ok(r) => r,
//         Err(e) => {
//             if let Some(e_str) = e.downcast_ref::<&str>() {
//                 let e_string: string_t = format!("Panicked with: {e_str}").into();
//                 unsafe { err_ptr.write(e_string) }
//             } else {
//                 unsafe { err_ptr.write("Panicked".into()) }
//             }
//             return SR_FATAL;
//         }
//     };

//     match res {
//         Ok(n) => n,
//         Err(e) => {
//             unsafe { err_ptr.write(e) }
//             SR_ERROR
//         }
//     }
// }

/// Execute a givel closure in an async context, which returns a result then catches panics and writes errors appropriately
fn with_surreal_async<'a, 'b, C, F>(db: &'a Surreal, err_ptr: *mut string_t, fun: C) -> c_int
where
    'a: 'b,
    C: FnOnce(&'a Surreal) -> F + 'b,
    F: std::future::Future<Output = Result<c_int, string_t>>,
{
    if db.ps.load(Ordering::Acquire) {
        std::process::abort()
    }
    let _guard = db.rt.enter();

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

pub trait SyncAssert: Sync {}
impl SyncAssert for Surreal {}
