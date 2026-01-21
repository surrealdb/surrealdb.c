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
use serde::{Deserialize, Serialize};
use stream::Stream;
use string::string_t;
use surrealdb::{
    engine::any::{self, Any},
    opt::Resource,
    opt::auth,
    opt::PatchOp,
    sql, Surreal as sdbSurreal, Value as apiValue,
};
use tokio::runtime::Runtime;
use types::result::ArrayResult;

use array::{Array, ArrayGen, MakeArray};
pub use types::*;
use utils::CStringExt2;
use value::{Object, Value};
use crate::credentials::{credentials_scope, credentials_access};

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
    /// authenticate with a token
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// const sr_string_t token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...";
    /// if (sr_authenticate(db, &err, token) < 0) {
    ///     printf("Failed to authenticate: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_authenticate"]
    pub extern "C" fn authenticate(
        db: &Surreal,
        err_ptr: *mut string_t,
        token: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let token = unsafe { CStr::from_ptr(token) }.to_str()?;
            surreal.db.authenticate(token).await?;
            Ok(0)
        })
    }

    // begin.rs
    /// begin a new transaction
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_begin(db, &err) < 0) {
    ///     printf("Failed to begin transaction: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_begin"]
    pub extern "C" fn begin(db: &Surreal, err_ptr: *mut string_t) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            surreal.db.query("BEGIN TRANSACTION").await?;
            Ok(0)
        })
    }

    // cancel.rs
    /// cancel the current transaction
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_cancel(db, &err) < 0) {
    ///     printf("Failed to cancel transaction: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_cancel"]
    pub extern "C" fn cancel(db: &Surreal, err_ptr: *mut string_t) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            surreal.db.query("CANCEL TRANSACTION").await?;
            Ok(0)
        })
    }

    // commit.rs
    /// commit the current transaction
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_commit(db, &err) < 0) {
    ///     printf("Failed to commit transaction: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_commit"]
    pub extern "C" fn commit(db: &Surreal, err_ptr: *mut string_t) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            surreal.db.query("COMMIT TRANSACTION").await?;
            Ok(0)
        })
    }

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
    /// delete a record or records
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *deleted;
    /// int len = sr_delete(db, &err, &deleted, "foo:bar");
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(deleted, len);
    /// ```
    #[export_name = "sr_delete"]
    pub extern "C" fn delete(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let res = match surreal
                .db
                .delete(Resource::from(resource))
                .await?
                .into_inner()
            {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // export.rs
    /// export database data to a file
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_export(db, &err, "backup.surql") < 0) {
    ///     printf("Export failed: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_export"]
    pub extern "C" fn export(
        db: &Surreal,
        err_ptr: *mut string_t,
        file_path: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let file_path = unsafe { CStr::from_ptr(file_path) }.to_str()?;
            surreal.db.export(file_path).await?;
            Ok(0)
        })
    }

    // health.rs
    /// check the health of the database server
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_health(db, &err) < 0) {
    ///     printf("Database unhealthy: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_health"]
    pub extern "C" fn health(db: &Surreal, err_ptr: *mut string_t) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            surreal.db.health().await?;
            Ok(0)
        })
    }

    // import.rs
    /// import database data from a file
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_import(db, &err, "backup.surql") < 0) {
    ///     printf("Import failed: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_import"]
    pub extern "C" fn import(
        db: &Surreal,
        err_ptr: *mut string_t,
        file_path: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let file_path = unsafe { CStr::from_ptr(file_path) }.to_str()?;
            surreal.db.import(file_path).await?;
            Ok(0)
        })
    }

    // insert.rs
    /// insert one or more records
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *inserted;
    /// sr_object_t *content = ...; // create content object
    /// int len = sr_insert(db, &err, &inserted, "foo", content);
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(inserted, len);
    /// ```
    #[export_name = "sr_insert"]
    pub extern "C" fn insert(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = match surreal
                .db
                .insert(Resource::from(resource))
                .content(content)
                .await?
                .into_inner()
            {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // insert_relation.rs
    /// Insert a relation between records
    ///
    /// The content object must contain 'in' and 'out' fields specifying the records to relate.
    /// Additional fields can be added as relation properties.
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *result;
    /// sr_object_t *content = sr_object_new();
    /// sr_object_insert_str(content, "in", "person:john");
    /// sr_object_insert_str(content, "out", "person:jane");
    /// sr_object_insert_str(content, "met", "2024-01-01");
    /// int len = sr_insert_relation(db, &err, &result, "knows", content);
    /// if (len < 0) {
    ///     printf("Failed to insert relation: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(result, len);
    /// ```
    #[export_name = "sr_insert_relation"]
    pub extern "C" fn insert_relation(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        table: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let table = unsafe { CStr::from_ptr(table) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = surreal
                .db
                .insert(Resource::from(table))
                .relation(content)
                .await?;

            let result = match res.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // run.rs
    /// Execute a SurrealDB function
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *result;
    /// sr_array_t args = ...; // create args array
    /// if (sr_run(db, &err, &result, "fn::my_function", &args) < 0) {
    ///     printf("Failed to run function: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(result, 1);
    /// ```
    #[export_name = "sr_run"]
    pub extern "C" fn run(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        function_name: *const c_char,
        args: *const Array,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let function_name = unsafe { CStr::from_ptr(function_name) }.to_str()?;
            
            let args_vec: Vec<sql::Value> = if args.is_null() {
                vec![]
            } else {
                let arr = unsafe { &*args };
                arr.as_slice().iter().cloned().map(|v| v.into()).collect()
            };

            let res: apiValue = surreal
                .db
                .run(function_name)
                .args(args_vec)
                .await?;

            let result_arr = match res.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result_arr.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // relate.rs
    /// Create a graph relation between two records
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *result;
    /// sr_object_t *content = sr_object_new();
    /// int len = sr_relate(db, &err, &result, "person:john", "knows", "person:jane", content);
    /// if (len < 0) {
    ///     printf("Failed to create relation: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(result, len);
    /// ```
    #[export_name = "sr_relate"]
    pub extern "C" fn relate(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        from: *const c_char,
        relation: *const c_char,
        to: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let from = unsafe { CStr::from_ptr(from) }.to_str()?;
            let relation = unsafe { CStr::from_ptr(relation) }.to_str()?;
            let to = unsafe { CStr::from_ptr(to) }.to_str()?;

            let query = if content.is_null() {
                format!("RELATE {from}->{relation}->{to}")
            } else {
                format!("RELATE {from}->{relation}->{to} CONTENT $content")
            };

            let mut q = surreal.db.query(&query);
            
            if !content.is_null() {
                let content_obj = sql::Object::from(unsafe { &*content }.clone());
                q = q.bind(("content", content_obj));
            }

            let mut res = q.await?;
            
            let result = match res.take::<apiValue>(0)?.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // invalidate.rs
    /// invalidate the current authentication session
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_invalidate(db, &err) < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_invalidate"]
    pub extern "C" fn invalidate(db: &Surreal, err_ptr: *mut string_t) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            surreal.db.invalidate().await?;
            Ok(0)
        })
    }

    // live.rs
    /// make a live selection
    /// if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
    /// which can be moved across threads but not aliased
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
    /// merge data into existing records
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *merged;
    /// sr_object_t *content = ...; // create content object
    /// int len = sr_merge(db, &err, &merged, "foo:bar", content);
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(merged, len);
    /// ```
    #[export_name = "sr_merge"]
    pub extern "C" fn merge(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = match surreal
                .db
                .update(Resource::from(resource))
                .merge(content)
                .await?
                .into_inner()
            {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // mod.rs

    // patch.rs
    /// Add a value at a JSON path using JSON Patch
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *patched;
    /// sr_value_t value = ...; // create value to add
    /// int len = sr_patch_add(db, &err, &patched, "person:john", "/tags/0", &value);
    /// if (len < 0) {
    ///     printf("Failed to patch: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(patched, len);
    /// ```
    #[export_name = "sr_patch_add"]
    pub extern "C" fn patch_add(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        path: *const c_char,
        value: *const Value,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;
            let value: sql::Value = unsafe { &*value }.clone().into();

            let res = surreal
                .db
                .update(Resource::from(resource))
                .patch(PatchOp::add(path, value))
                .await?;

            let result = match res.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Remove a value at a JSON path using JSON Patch
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *patched;
    /// int len = sr_patch_remove(db, &err, &patched, "person:john", "/temporary_field");
    /// if (len < 0) {
    ///     printf("Failed to patch: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(patched, len);
    /// ```
    #[export_name = "sr_patch_remove"]
    pub extern "C" fn patch_remove(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        path: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;

            let res = surreal
                .db
                .update(Resource::from(resource))
                .patch(PatchOp::remove(path))
                .await?;

            let result = match res.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Replace a value at a JSON path using JSON Patch
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *patched;
    /// sr_value_t value = ...; // create new value
    /// int len = sr_patch_replace(db, &err, &patched, "person:john", "/name", &value);
    /// if (len < 0) {
    ///     printf("Failed to patch: %s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(patched, len);
    /// ```
    #[export_name = "sr_patch_replace"]
    pub extern "C" fn patch_replace(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        path: *const c_char,
        value: *const Value,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;
            let value: sql::Value = unsafe { &*value }.clone().into();

            let res = surreal
                .db
                .update(Resource::from(resource))
                .patch(PatchOp::replace(path, value))
                .await?;

            let result = match res.into_inner() {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

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
    /// writes values to *res_ptr, and returns the number of values
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

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }
    // set.rs
    /// set a variable for the current session
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *value = ...; // create value
    /// if (sr_set(db, &err, "my_var", value) < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_set"]
    pub extern "C" fn set(
        db: &Surreal,
        err_ptr: *mut string_t,
        key: *const c_char,
        value: *const Value,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let key = unsafe { CStr::from_ptr(key) }.to_str()?;
            let value: sql::Value = unsafe { &*value }.clone().into();
            
            surreal.db.set(key, value).await?;
            Ok(0)
        })
    }

    // signin.rs
    /// Sign in utilizing the surreal authentication types.
    ///
    /// Used to provide credentials to a db for access permissions, either root or scoped.
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    ///
    /// sr_credentials_scope scope = sr_credentials_scope::ROOT;
    /// const sr_string_t user = "<user>";
    /// // SHOULD NEVER BE HARDCODED
    /// const sr_string_t password = "<password>;
    /// sr_credentials creds = sr_credentials {
    ///     .username = user,
    ///     .password = pass,
    /// };
    ///
    /// if (sr_signin(db, &err, &scope, &creds, nullptr) < 0) {
    ///     printf("Failed to authenticate credentials: %s", err);
    ///     return 1;
    /// }
    /// ```
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    ///
    /// sr_credentials_scope scope = sr_credentials_scope::DATABASE;
    /// const sr_string_t user = "<user>";
    /// // SHOULD NEVER BE HARDCODED
    /// const sr_string_t password = "<password>;
    /// sr_credentials creds = sr_credentials {
    ///     .username = user,
    ///     .password = pass,
    /// };
    /// sr_string_t namespace_ = "testing";
    /// sr_string_t db_name = "perf-test";
    /// sr_credentials_access details = sr_credentials_access {
    ///     .namespace_ = namespace_,
    ///     .database = db_name,
    ///     .access = nullptr,
    /// };
    ///
    /// if (sr_signin(db, &err, &scope, &creds, &details) < 0) {
    ///     printf("Failed to authenticate credentials: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_signin"]
    pub extern "C" fn signin(
        db: &Surreal,
        err_ptr: *mut string_t,
        scope: &credentials_scope,
        creds: &credentials::credentials,
        details: *const credentials_access
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let user = unsafe { CStr::from_ptr(creds.username.0).to_str()? };
            let pass = unsafe { CStr::from_ptr(creds.password.0).to_str()? };

            let mut ns = "";
            let mut db = "";
            let mut ac = "";

            if !details.is_null() {
                let details = unsafe { &*details };

                if !details.namespace.0.is_null() {
                    ns = unsafe { CStr::from_ptr(details.namespace.0).to_str()? };
                }

                if !details.database.0.is_null() {
                    db = unsafe { CStr::from_ptr(details.database.0).to_str()? };
                }

                if !details.access.0.is_null() {
                    ac = unsafe { CStr::from_ptr(details.access.0).to_str()? };
                }
            }

            match scope {
                credentials_scope::ROOT => {
                    let login = auth::Root {
                        username: user,
                        password: pass,
                    };
                    let _res = surreal.db.signin(login).await?;
                }
                credentials_scope::NAMESPACE => {
                    if ns.is_empty() {
                        Err("Namespace must be provided.")?
                    }

                    let login = auth::Namespace {
                        namespace: ns,
                        username: user,
                        password: pass,
                    };

                    let _res = surreal.db.signin(login).await?;
                }
                credentials_scope::DATABASE => {
                    if ns.is_empty() {
                        Err("Namespace must be provided.")?
                    }
                    if db.is_empty() {
                        Err("Database must be provided.")?
                    }

                    let login = auth::Database {
                        namespace: ns,
                        database: db,
                        username: user,
                        password: pass,
                    };

                    let _res = surreal.db.signin(login).await?;
                }
                credentials_scope::RECORD => {
                    if ns.is_empty() {
                        Err("Namespace must be provided.")?
                    }
                    if db.is_empty() {
                        Err("Database must be provided.")?
                    }
                    if ac.is_empty() {
                        Err("Access method must be provided.")?
                    }

                    #[derive(Debug, Serialize, Deserialize)]
                    struct CredsInner {
                        username: String,
                        password: String,
                    }

                    let creds_inner = CredsInner {
                        username: user.to_string(),
                        password: pass.to_string()
                    };

                    let login = auth::Record {
                        namespace: ns,
                        database: db,
                        access: ac,
                        params: creds_inner,
                    };

                    let _res = surreal.db.signin(login).await?;
                }
            };
            Ok(0)
        })
    }



    // signup.rs
    /// Sign up a new user with credentials
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_credentials_scope scope = sr_credentials_scope::RECORD;
    /// const sr_string_t user = "newuser";
    /// const sr_string_t password = "password123";
    /// sr_credentials creds = sr_credentials {
    ///     .username = user,
    ///     .password = password,
    /// };
    /// sr_string_t namespace_ = "test";
    /// sr_string_t db_name = "test";
    /// sr_string_t access = "user";
    /// sr_credentials_access details = sr_credentials_access {
    ///     .namespace_ = namespace_,
    ///     .database = db_name,
    ///     .access = access,
    /// };
    ///
    /// if (sr_signup(db, &err, &scope, &creds, &details) < 0) {
    ///     printf("Failed to sign up: %s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_signup"]
    pub extern "C" fn signup(
        db: &Surreal,
        err_ptr: *mut string_t,
        scope: &credentials_scope,
        creds: &credentials::credentials,
        details: *const credentials_access
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let user = unsafe { CStr::from_ptr(creds.username.0).to_str()? };
            let pass = unsafe { CStr::from_ptr(creds.password.0).to_str()? };

            let mut ns = "";
            let mut db = "";
            let mut ac = "";

            if !details.is_null() {
                let details = unsafe { &*details };

                if !details.namespace.0.is_null() {
                    ns = unsafe { CStr::from_ptr(details.namespace.0).to_str()? };
                }

                if !details.database.0.is_null() {
                    db = unsafe { CStr::from_ptr(details.database.0).to_str()? };
                }

                if !details.access.0.is_null() {
                    ac = unsafe { CStr::from_ptr(details.access.0).to_str()? };
                }
            }

            match scope {
                credentials_scope::ROOT => {
                    Err("Cannot signup as ROOT user")?
                }
                credentials_scope::NAMESPACE => {
                    Err("Namespace scope does not support signup. Use RECORD scope instead.")?
                }
                credentials_scope::DATABASE => {
                    Err("Database scope does not support signup. Use RECORD scope instead.")?
                }
                credentials_scope::RECORD => {
                    if ns.is_empty() {
                        Err("Namespace must be provided.")?
                    }
                    if db.is_empty() {
                        Err("Database must be provided.")?
                    }
                    if ac.is_empty() {
                        Err("Access method must be provided.")?
                    }

                    #[derive(Debug, Serialize, Deserialize)]
                    struct CredsInner {
                        username: String,
                        password: String,
                    }

                    let creds_inner = CredsInner {
                        username: user.to_string(),
                        password: pass.to_string()
                    };

                    let signup = auth::Record {
                        namespace: ns,
                        database: db,
                        access: ac,
                        params: creds_inner,
                    };

                    let _res = surreal.db.signup(signup).await?;
                }
            };
            Ok(0)
        })
    }

    // unset.rs
    /// unset a variable from the current session
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// if (sr_unset(db, &err, "my_var") < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_unset"]
    pub extern "C" fn unset(
        db: &Surreal,
        err_ptr: *mut string_t,
        key: *const c_char,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let key = unsafe { CStr::from_ptr(key) }.to_str()?;
            surreal.db.unset(key).await?;
            Ok(0)
        })
    }

    // update.rs
    /// update records with new content
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *updated;
    /// sr_object_t *content = ...; // create content object
    /// int len = sr_update(db, &err, &updated, "foo:bar", content);
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(updated, len);
    /// ```
    #[export_name = "sr_update"]
    pub extern "C" fn update(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = match surreal
                .db
                .update(Resource::from(resource))
                .content(content)
                .await?
                .into_inner()
            {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    // upsert.rs
    /// upsert (insert or update) records
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_value_t *upserted;
    /// sr_object_t *content = ...; // create content object
    /// int len = sr_upsert(db, &err, &upserted, "foo:bar", content);
    /// if (len < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// sr_free_arr(upserted, len);
    /// ```
    #[export_name = "sr_upsert"]
    pub extern "C" fn upsert(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut Value,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sql::Object::from(unsafe { &*content }.clone());

            let res = match surreal
                .db
                .upsert(Resource::from(resource))
                .content(content)
                .await?
                .into_inner()
            {
                sql::Value::Array(a) => Array::from(a),
                v => Array::from(vec![v.into()]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

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

/// Execute a given closure in an async context, which returns a result then catches panics and writes errors appropriately
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
