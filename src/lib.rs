#![recursion_limit = "256"]

pub mod opts;
pub mod rpc;
pub mod types;
pub mod utils;

use std::{
    ffi::{c_char, c_int, CStr},
    future::IntoFuture,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use stream::Stream;
use string::string_t;
use surrealdb::{
    engine::any::{self, Any},
    opt::Resource,
    opt::auth,
    opt::PatchOp,
    Surreal as sdbSurreal,
};
use surrealdb::types::{Value as sdbValue, Object as sdbObject, RecordId, RecordIdKey};

fn parse_resource(s: &str) -> Resource {
    if let Some((table, id)) = s.split_once(':') {
        if !table.is_empty() && !id.is_empty() {
            let record_id = RecordId {
                table: table.to_string().into(),
                key: RecordIdKey::String(id.to_string()),
            };
            return Resource::RecordId(record_id);
        }
    }
    Resource::from(s)
}
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

/// Safely write an error message to an error pointer
/// 
/// If `err_ptr` is null, the error is silently ignored.
#[inline]
fn write_error(err_ptr: *mut string_t, msg: impl Into<string_t>) {
    if !err_ptr.is_null() {
        unsafe { err_ptr.write(msg.into()) };
    }
}

/// Macro to validate that a pointer is not null
/// 
/// If the pointer is null, writes an error message and returns SR_ERROR.
macro_rules! check_null {
    ($ptr:expr, $err_ptr:expr, $msg:expr) => {
        if $ptr.is_null() {
            write_error($err_ptr, $msg);
            return SR_ERROR;
        }
    };
}

/// The object representing a Surreal connection
///
/// It is safe to be referenced from multiple threads
/// If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
/// (use will cause the program to abort)
///
/// should be freed with sr_surreal_disconnect
pub struct Surreal {
    db: sdbSurreal<Any>,
    rt: Arc<Runtime>,
    ps: AtomicBool,
}

impl Surreal {
    /// Connects to a local, remote, or embedded database
    ///
    /// If any function returns SR_FATAL, the connection is poisoned and must not be used
    /// (except to drop). Continued use will cause the program to abort.
    ///
    /// # Safety
    ///
    /// - `err_ptr` must be a valid pointer or null (errors ignored if null)
    /// - `surreal_ptr` must be a valid pointer to receive the connection handle
    /// - `endpoint` must be a valid null-terminated UTF-8 string
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
        check_null!(surreal_ptr, err_ptr, "surreal_ptr is null");
        check_null!(endpoint, err_ptr, "endpoint is null");

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
                Err(e) => return Err(e.to_string().into()),
            };

            Ok(Surreal {
                db,
                rt: Arc::new(rt),
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

    /// Disconnect a database connection
    ///
    /// The Surreal object must not be used after this function has been called.
    /// Any object allocations will still be valid and should be freed using the appropriate function.
    ///
    /// Note: Stream objects should be killed before disconnection to ensure proper cleanup.
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

    /// Clone a database connection
    ///
    /// Creates a new independent Surreal handle that shares the same underlying
    /// database engine as the original. The cloned handle has its own Tokio runtime
    /// and independent namespace/database state.
    ///
    /// This is equivalent to Rust SDK's `db.clone()` — enabling multiple independent
    /// sessions on a single embedded (surrealkv://) connection without file locking issues.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `clone_ptr` must be a valid pointer to receive the cloned handle
    /// - The original `db` must NOT be disconnected while clones are in use
    /// - Each clone must be freed with `sr_surreal_disconnect`
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_surreal_t *clone;
    /// sr_string_t err;
    ///
    /// // Connect once
    /// if (sr_connect(&err, &db, "surrealkv://test.skv") < 0) {
    ///     printf("error: %s\n", err);
    ///     return 1;
    /// }
    ///
    /// // Clone for independent session
    /// if (sr_clone(db, &err, &clone) < 0) {
    ///     printf("error: %s\n", err);
    ///     return 1;
    /// }
    ///
    /// // Each handle can use different NS/DB independently
    /// sr_use_ns(db, &err, "ns1");
    /// sr_use_db(db, &err, "db1");
    /// sr_use_ns(clone, &err, "ns2");
    /// sr_use_db(clone, &err, "db2");
    ///
    /// // Queries run concurrently on separate sessions
    /// // ...
    ///
    /// sr_surreal_disconnect(clone);
    /// sr_surreal_disconnect(db);
    /// ```
    #[export_name = "sr_clone"]
    pub extern "C" fn clone_connection(
        db: &Surreal,
        err_ptr: *mut string_t,
        clone_ptr: *mut *mut Surreal,
    ) -> c_int {
        check_null!(clone_ptr, err_ptr, "clone_ptr is null");

        let res: Result<Result<Surreal, string_t>, _> = catch_unwind(AssertUnwindSafe(|| {
            if db.ps.load(Ordering::Acquire) {
                return Err("connection is poisoned".into());
            }

            // Clone the underlying Surreal<Any> — shares the same engine
            let cloned_db = db.db.clone();

            // Share the parent's Tokio runtime (Arc::clone is ~0 cost)
            let shared_rt = Arc::clone(&db.rt);

            Ok(Surreal {
                db: cloned_db,
                rt: shared_rt,
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
                unsafe { clone_ptr.write(Box::leak(boxed)) }
                1
            }
            Err(e) => {
                unsafe { err_ptr.write(e) }
                SR_ERROR
            }
        }
    }

    /// Authenticate with a token
    ///
    /// Authenticates the current connection with a JWT token.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `token` must be a valid null-terminated UTF-8 string
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
        check_null!(token, err_ptr, "token is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let token = unsafe { CStr::from_ptr(token) }.to_str()?;
            surreal.db.authenticate(token).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Begin a new transaction
    ///
    /// Starts a new database transaction.
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
            surreal.db.query("BEGIN TRANSACTION").await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Cancel the current transaction
    ///
    /// Cancels and rolls back the current database transaction.
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
            surreal.db.query("CANCEL TRANSACTION").await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Commit the current transaction
    ///
    /// Commits and finalizes the current database transaction.
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
            surreal.db.query("COMMIT TRANSACTION").await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Create a record
    ///
    /// Creates a new record in the specified resource with the given content.
    /// The resource can be a table name (e.g., "user") for auto-generated IDs,
    /// or a specific record ID (e.g., "user:john").
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` may be null (result will be discarded)
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
    #[export_name = "sr_create"]
    pub extern "C" fn create(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut &mut Object,
        resource: *const c_char,
        content: *const Object,
    ) -> c_int {
        check_null!(resource, err_ptr, "resource is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let query = format!("CREATE {} CONTENT $content", resource);
            let mut res = surreal.db.query(&query).bind(("content", content)).await
                .map_err(|e| string_t::from(e.to_string()))?;
            
            let val: sdbValue = res.take(0).map_err(|e| string_t::from(e.to_string()))?;
            let obj = match val {
                sdbValue::Array(arr) if !arr.is_empty() => {
                    match arr.into_iter().next().unwrap() {
                        sdbValue::Object(o) => o,
                        other => {
                            return Err(format!(
                                "Expected object as return type of create, but found: {other:?}"
                            )
                            .into())
                        }
                    }
                }
                sdbValue::Object(o) => o,
                other => {
                    return Err(format!(
                        "Expected object as return type of create, but found: {other:?}"
                    )
                    .into())
                }
            };
            if !res_ptr.is_null() {
                let boxed = Box::new(Object::from(obj));
                unsafe { res_ptr.write(Box::leak(boxed)) }
                Ok(1)
            } else {
                Ok(0)
            }
        })
    }

    /// Delete a record or records
    ///
    /// Deletes records from the specified resource.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let val: sdbValue = surreal
                .db
                .delete(parse_resource(resource))
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Export database data to a file
    ///
    /// Exports all data from the current namespace and database to a file.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `file_path` must be a valid null-terminated UTF-8 string
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
        check_null!(file_path, err_ptr, "file_path is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let file_path = unsafe { CStr::from_ptr(file_path) }.to_str()?;
            surreal.db.export(file_path).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Check the health of the database server
    ///
    /// Performs a health check on the database connection.
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
            surreal.db.health().await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Import database data from a file
    ///
    /// Imports data from a file into the current namespace and database.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `file_path` must be a valid null-terminated UTF-8 string
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
        check_null!(file_path, err_ptr, "file_path is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let file_path = unsafe { CStr::from_ptr(file_path) }.to_str()?;
            surreal.db.import(file_path).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Insert one or more records
    ///
    /// Inserts records into the specified resource with the given content.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let val: sdbValue = surreal
                .db
                .insert(parse_resource(resource))
                .content(content)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Insert a relation between records
    ///
    /// Creates a relation record in a relation table.
    ///
    /// The content object must contain 'in' and 'out' fields specifying the records to relate.
    /// Additional fields can be added as relation properties.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `table` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(table, err_ptr, "table is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let table = unsafe { CStr::from_ptr(table) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let val: sdbValue = surreal
                .db
                .insert(Resource::from(table))
                .relation(content)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let result = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Execute a SurrealDB function
    ///
    /// Runs a custom or built-in SurrealDB function with the specified arguments.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `function_name` must be a valid null-terminated UTF-8 string
    /// - `args` may be null (empty arguments)
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(function_name, err_ptr, "function_name is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let function_name = unsafe { CStr::from_ptr(function_name) }.to_str()?;
            
            let args_vec: Vec<sdbValue> = if args.is_null() {
                vec![]
            } else {
                let arr = unsafe { &*args };
                arr.as_slice().iter().cloned().map(|v| sdbValue::from(v)).collect()
            };

            let res: sdbValue = surreal
                .db
                .run(function_name)
                .args(args_vec)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let result_arr = match res {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result_arr.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Create a graph relation between two records
    ///
    /// Establishes a directed relation from one record to another through a relation table.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `from` must be a valid null-terminated UTF-8 string
    /// - `relation` must be a valid null-terminated UTF-8 string
    /// - `to` must be a valid null-terminated UTF-8 string
    /// - `content` may be null (no content will be added)
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(from, err_ptr, "from is null");
        check_null!(relation, err_ptr, "relation is null");
        check_null!(to, err_ptr, "to is null");
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
                let content_obj = sdbObject::from(unsafe { &*content }.clone());
                q = q.bind(("content", content_obj));
            }

            let mut res = q.await.map_err(|e| string_t::from(e.to_string()))?;
            
            let val: sdbValue = res.take(0).map_err(|e| string_t::from(e.to_string()))?;
            let result = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Invalidate the current authentication session
    ///
    /// Clears the current authentication for the connection.
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
            surreal.db.invalidate().await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Kill a live query by its UUID string
    ///
    /// Terminates an active live query subscription.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `query_id` must be a valid null-terminated UTF-8 string
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// const char *query_id = "..."; // UUID string from live query
    /// if (sr_kill(db, &err, query_id) < 0) {
    ///     printf("%s", err);
    ///     return 1;
    /// }
    /// ```
    #[export_name = "sr_kill"]
    pub extern "C" fn kill(db: &Surreal, err_ptr: *mut string_t, query_id: *const c_char) -> c_int {
        check_null!(query_id, err_ptr, "query_id is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let uuid_str = unsafe { CStr::from_ptr(query_id) }.to_str()?;
            let query = format!("KILL u'{}'", uuid_str);
            surreal.db.query(query).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Make a live selection
    ///
    /// Creates a live query subscription that streams changes.
    /// if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
    /// which can be moved across threads but not aliased
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `stream_ptr` must be a valid pointer to receive the stream
    /// - `resource` must be a valid null-terminated UTF-8 string
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
        check_null!(stream_ptr, err_ptr, "stream_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        use surrealdb::method::Stream as sdbStreamType;
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let stream_inner: sdbStreamType<sdbValue> =
                surreal.db.select(parse_resource(resource)).live().await
                    .map_err(|e| string_t::from(e.to_string()))?;

            let stream_boxed = Box::new(Stream::new(stream_inner, surreal.rt.handle().clone()));

            unsafe { stream_ptr.write(Box::leak(stream_boxed)) };

            Ok(1)
        })
    }

    /// Merge data into existing records
    ///
    /// Merges the provided content into existing records, preserving unmodified fields.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let val: sdbValue = surreal
                .db
                .update(parse_resource(resource))
                .merge(content)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Add a value at a JSON path using JSON Patch
    ///
    /// Applies a JSON Patch add operation to the specified resource.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `path` must be a valid null-terminated UTF-8 string
    /// - `value` must be a valid pointer to a Value
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(path, err_ptr, "path is null");
        check_null!(value, err_ptr, "value is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;
            let value: sdbValue = unsafe { &*value }.clone().into();

            let val: sdbValue = surreal
                .db
                .update(parse_resource(resource))
                .patch(PatchOp::add(path, value))
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let result = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Remove a value at a JSON path using JSON Patch
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `path` must be a valid null-terminated UTF-8 string
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(path, err_ptr, "path is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;

            let val: sdbValue = surreal
                .db
                .update(parse_resource(resource))
                .patch(PatchOp::remove(path))
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let result = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Replace a value at a JSON path using JSON Patch
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `path` must be a valid null-terminated UTF-8 string
    /// - `value` must be a valid pointer to a Value
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(path, err_ptr, "path is null");
        check_null!(value, err_ptr, "value is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let path = unsafe { CStr::from_ptr(path) }.to_str()?;
            let value: sdbValue = unsafe { &*value }.clone().into();

            let val: sdbValue = surreal
                .db
                .update(parse_resource(resource))
                .patch(PatchOp::replace(path, value))
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let result = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = result.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Execute a SurrealQL query
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `query` must be a valid null-terminated UTF-8 string
    /// - `vars` may be null (no variables bound)
    #[export_name = "sr_query"]
    pub extern "C" fn query(
        db: &Surreal,
        err_ptr: *mut string_t,
        res_ptr: *mut *mut ArrayResult,
        query: *const c_char,
        vars: *const Object,
    ) -> c_int {
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(query, err_ptr, "query is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let query = unsafe { CStr::from_ptr(query) }.to_str()?;
            let vars: sdbObject = match vars.is_null() {
                true => sdbObject::default(),
                false => unsafe { &*vars }.clone().into(),
            };

            let mut res = surreal.db.query(query).bind(vars).await
                .map_err(|e| string_t::from(e.to_string()))?;
            let res_len = res.num_statements();

            let mut acc = Vec::with_capacity(res_len);
            for index in 0..res_len {
                let api_res: Result<sdbValue, _> = res.take(index);
                let arr_res = match api_res {
                    Ok(sdbValue::Array(arr)) => {
                        let a = arr.into();
                        ArrayResult::ok(a)
                    }
                    Ok(val) => {
                        let arr: Array = vec![Value::from(val)].into();
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

    /// Select a resource
    ///
    /// Selects records from the specified resource (table or record ID).
    ///
    /// can be used to select everything from a table or a single record
    /// writes values to *res_ptr, and returns the number of values
    /// result values are allocated by Surreal and must be freed with sr_free_arr
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;

            let val: sdbValue = surreal
                .db
                .select(parse_resource(resource))
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Set a variable for the current session
    ///
    /// Defines a session variable that can be referenced in queries.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `key` must be a valid null-terminated UTF-8 string
    /// - `value` must be a valid pointer to a Value
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
        check_null!(key, err_ptr, "key is null");
        check_null!(value, err_ptr, "value is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let key = unsafe { CStr::from_ptr(key) }.to_str()?;
            let value: sdbValue = unsafe { &*value }.clone().into();
            
            surreal.db.set(key, value).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Sign in utilizing the surreal authentication types
    ///
    /// Authenticates with the database using the provided credentials.
    ///
    /// Used to provide credentials to a db for access permissions, either root or scoped.
    /// Returns the JWT token via token_ptr if not null.
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_string_t token;
    ///
    /// sr_credentials_scope scope = sr_credentials_scope::ROOT;
    /// const sr_string_t user = "<user>";
    /// // SHOULD NEVER BE HARDCODED
    /// const sr_string_t password = "<password>";
    /// sr_credentials creds = sr_credentials {
    ///     .username = user,
    ///     .password = pass,
    /// };
    ///
    /// if (sr_signin(db, &err, &token, &scope, &creds, nullptr, nullptr) < 0) {
    ///     printf("Failed to authenticate credentials: %s", err);
    ///     return 1;
    /// }
    /// // token now contains the JWT
    /// sr_free_string(token);
    /// ```
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_string_t token;
    ///
    /// sr_credentials_scope scope = sr_credentials_scope::DATABASE;
    /// const sr_string_t user = "<user>";
    /// // SHOULD NEVER BE HARDCODED
    /// const sr_string_t password = "<password>";
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
    /// if (sr_signin(db, &err, &token, &scope, &creds, &details, nullptr) < 0) {
    ///     printf("Failed to authenticate credentials: %s", err);
    ///     return 1;
    /// }
    /// ```
    /// For RECORD scope with custom params:
    /// ```c
    /// sr_object_t params = sr_object_new();
    /// sr_object_insert_str(&params, "email", "user@example.com");
    /// sr_object_insert_str(&params, "password", "secret");
    /// if (sr_signin(db, &err, &token, &scope, nullptr, &details, &params) < 0) {
    ///     // handle error
    /// }
    /// ```
    #[export_name = "sr_signin"]
    pub extern "C" fn signin(
        db: &Surreal,
        err_ptr: *mut string_t,
        token_ptr: *mut string_t,
        scope: &credentials_scope,
        creds: *const credentials::credentials,
        details: *const credentials_access,
        params: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let mut user = "";
            let mut pass = "";
            
            if !creds.is_null() {
                let creds = unsafe { &*creds };
                if !creds.username.0.is_null() {
                    user = unsafe { CStr::from_ptr(creds.username.0).to_str()? };
                }
                if !creds.password.0.is_null() {
                    pass = unsafe { CStr::from_ptr(creds.password.0).to_str()? };
                }
            }

            let mut ns = "";
            let mut db_name = "";
            let mut ac = "";

            if !details.is_null() {
                let details = unsafe { &*details };

                if !details.namespace.0.is_null() {
                    ns = unsafe { CStr::from_ptr(details.namespace.0).to_str()? };
                }

                if !details.database.0.is_null() {
                    db_name = unsafe { CStr::from_ptr(details.database.0).to_str()? };
                }

                if !details.access.0.is_null() {
                    ac = unsafe { CStr::from_ptr(details.access.0).to_str()? };
                }
            }

            let token: String = match scope {
                credentials_scope::ROOT => {
                    let login = auth::Root {
                        username: user.to_string(),
                        password: pass.to_string(),
                    };
                    let jwt = surreal.db.signin(login).await.map_err(|e| string_t::from(e.to_string()))?;
                    jwt.access.into_insecure_token()
                }
                credentials_scope::NAMESPACE => {
                    if ns.is_empty() {
                        return Err("Namespace must be provided.".into());
                    }

                    let login = auth::Namespace {
                        namespace: ns.to_string(),
                        username: user.to_string(),
                        password: pass.to_string(),
                    };

                    let jwt = surreal.db.signin(login).await.map_err(|e| string_t::from(e.to_string()))?;
                    jwt.access.into_insecure_token()
                }
                credentials_scope::DATABASE => {
                    if ns.is_empty() {
                        return Err("Namespace must be provided.".into());
                    }
                    if db_name.is_empty() {
                        return Err("Database must be provided.".into());
                    }

                    let login = auth::Database {
                        namespace: ns.to_string(),
                        database: db_name.to_string(),
                        username: user.to_string(),
                        password: pass.to_string(),
                    };

                    let jwt = surreal.db.signin(login).await.map_err(|e| string_t::from(e.to_string()))?;
                    jwt.access.into_insecure_token()
                }
                credentials_scope::RECORD => {
                    if ns.is_empty() {
                        return Err("Namespace must be provided.".into());
                    }
                    if db_name.is_empty() {
                        return Err("Database must be provided.".into());
                    }
                    if ac.is_empty() {
                        return Err("Access method must be provided.".into());
                    }

                    let record_params: sdbObject = if !params.is_null() {
                        unsafe { &*params }.clone().into()
                    } else {
                        let mut obj = sdbObject::default();
                        obj.insert("username".to_string(), sdbValue::String(user.to_string()));
                        obj.insert("password".to_string(), sdbValue::String(pass.to_string()));
                        obj
                    };

                    let login = auth::Record {
                        namespace: ns.to_string(),
                        database: db_name.to_string(),
                        access: ac.to_string(),
                        params: record_params,
                    };

                    let jwt = surreal.db.signin(login).await.map_err(|e| string_t::from(e.to_string()))?;
                    jwt.access.into_insecure_token()
                }
            };
            
            if !token_ptr.is_null() {
                unsafe { *token_ptr = token.to_string_t(); }
            }
            
            Ok(0)
        })
    }

    /// Sign up a new user with credentials
    ///
    /// Registers a new user account with the database.
    /// Returns the JWT token via token_ptr if not null.
    /// Only RECORD scope is supported for signup.
    ///
    /// # Examples
    ///
    /// ```c
    /// sr_surreal_t *db;
    /// sr_string_t err;
    /// sr_string_t token;
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
    /// if (sr_signup(db, &err, &token, &scope, &creds, &details, nullptr) < 0) {
    ///     printf("Failed to sign up: %s", err);
    ///     return 1;
    /// }
    /// // token now contains the JWT
    /// sr_free_string(token);
    /// ```
    /// For custom params:
    /// ```c
    /// sr_object_t params = sr_object_new();
    /// sr_object_insert_str(&params, "email", "user@example.com");
    /// sr_object_insert_str(&params, "password", "secret");
    /// if (sr_signup(db, &err, &token, &scope, nullptr, &details, &params) < 0) {
    ///     // handle error
    /// }
    /// ```
    #[export_name = "sr_signup"]
    pub extern "C" fn signup(
        db: &Surreal,
        err_ptr: *mut string_t,
        token_ptr: *mut string_t,
        scope: &credentials_scope,
        creds: *const credentials::credentials,
        details: *const credentials_access,
        params: *const Object,
    ) -> c_int {
        with_surreal_async(db, err_ptr, |surreal| async {
            let mut user = "";
            let mut pass = "";
            
            if !creds.is_null() {
                let creds = unsafe { &*creds };
                if !creds.username.0.is_null() {
                    user = unsafe { CStr::from_ptr(creds.username.0).to_str()? };
                }
                if !creds.password.0.is_null() {
                    pass = unsafe { CStr::from_ptr(creds.password.0).to_str()? };
                }
            }

            let mut ns = "";
            let mut db_name = "";
            let mut ac = "";

            if !details.is_null() {
                let details = unsafe { &*details };

                if !details.namespace.0.is_null() {
                    ns = unsafe { CStr::from_ptr(details.namespace.0).to_str()? };
                }

                if !details.database.0.is_null() {
                    db_name = unsafe { CStr::from_ptr(details.database.0).to_str()? };
                }

                if !details.access.0.is_null() {
                    ac = unsafe { CStr::from_ptr(details.access.0).to_str()? };
                }
            }

            let token: String = match scope {
                credentials_scope::ROOT => {
                    return Err("Cannot signup as ROOT user".into());
                }
                credentials_scope::NAMESPACE => {
                    return Err("Namespace scope does not support signup. Use RECORD scope instead.".into());
                }
                credentials_scope::DATABASE => {
                    return Err("Database scope does not support signup. Use RECORD scope instead.".into());
                }
                credentials_scope::RECORD => {
                    if ns.is_empty() {
                        return Err("Namespace must be provided.".into());
                    }
                    if db_name.is_empty() {
                        return Err("Database must be provided.".into());
                    }
                    if ac.is_empty() {
                        return Err("Access method must be provided.".into());
                    }

                    let record_params: sdbObject = if !params.is_null() {
                        unsafe { &*params }.clone().into()
                    } else {
                        let mut obj = sdbObject::default();
                        obj.insert("username".to_string(), sdbValue::String(user.to_string()));
                        obj.insert("password".to_string(), sdbValue::String(pass.to_string()));
                        obj
                    };

                    let signup = auth::Record {
                        namespace: ns.to_string(),
                        database: db_name.to_string(),
                        access: ac.to_string(),
                        params: record_params,
                    };

                    let jwt = surreal.db.signup(signup).await.map_err(|e| string_t::from(e.to_string()))?;
                    jwt.access.into_insecure_token()
                }
            };
            
            if !token_ptr.is_null() {
                unsafe { *token_ptr = token.to_string_t(); }
            }
            
            Ok(0)
        })
    }

    /// Unset a variable from the current session
    ///
    /// Removes a previously defined session variable.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `key` must be a valid null-terminated UTF-8 string
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
        check_null!(key, err_ptr, "key is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let key = unsafe { CStr::from_ptr(key) }.to_str()?;
            surreal.db.unset(key).await.map_err(|e| string_t::from(e.to_string()))?;
            Ok(0)
        })
    }

    /// Update records with new content
    ///
    /// Replaces the content of existing records with new data.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let val: sdbValue = surreal
                .db
                .update(parse_resource(resource))
                .content(content)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Upsert (insert or update) records
    ///
    /// Creates records if they don't exist, or updates them if they do.
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the result array
    /// - `resource` must be a valid null-terminated UTF-8 string
    /// - `content` must be a valid pointer to an Object
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        check_null!(resource, err_ptr, "resource is null");
        check_null!(content, err_ptr, "content is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let resource = unsafe { CStr::from_ptr(resource) }.to_str()?;
            let content = sdbObject::from(unsafe { &*content }.clone());

            let val: sdbValue = surreal
                .db
                .upsert(parse_resource(resource))
                .content(content)
                .await
                .map_err(|e| string_t::from(e.to_string()))?;

            let res = match val {
                sdbValue::Array(a) => Array::from(a),
                v => Array::from(vec![Value::from(v)]),
            };

            let ArrayGen { ptr, len } = res.into();
            unsafe { res_ptr.write(ptr) }

            Ok(len as c_int)
        })
    }

    /// Select database
    ///
    /// Sets the database to use for subsequent operations.
    /// NOTE: namespace must be selected first with sr_use_ns
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `db_name` must be a valid null-terminated UTF-8 string
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
    pub extern "C" fn use_db(db: &Surreal, err_ptr: *mut string_t, db_name: *const c_char) -> c_int {
        check_null!(db_name, err_ptr, "db_name is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let db_name = unsafe { CStr::from_ptr(db_name) }.to_str()?;

            surreal.db.use_db(db_name).await.map_err(|e| string_t::from(e.to_string()))?;

            Ok(0)
        })
    }

    /// Select namespace
    ///
    /// Sets the namespace to use for subsequent operations.
    /// NOTE: database must be selected before use with sr_use_db
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `ns_name` must be a valid null-terminated UTF-8 string
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
    pub extern "C" fn use_ns(db: &Surreal, err_ptr: *mut string_t, ns_name: *const c_char) -> c_int {
        check_null!(ns_name, err_ptr, "ns_name is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let ns_name = unsafe { CStr::from_ptr(ns_name) }.to_str()?;

            surreal.db.use_ns(ns_name).await.map_err(|e| string_t::from(e.to_string()))?;

            Ok(0)
        })
    }

    /// Returns the database version
    ///
    /// Retrieves the version string of the connected SurrealDB server.
    /// NOTE: version is allocated in Surreal and must be freed with sr_free_string
    ///
    /// # Safety
    ///
    /// - `db` must be a valid pointer to a Surreal connection
    /// - `err_ptr` must be a valid pointer or null
    /// - `res_ptr` must be a valid pointer to receive the version string
    ///
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
        check_null!(res_ptr, err_ptr, "res_ptr is null");
        with_surreal_async(db, err_ptr, |surreal| async {
            let res = surreal.db.version().await.map_err(|e| string_t::from(e.to_string()))?;
            let res_string = res.to_string();
            let len = res_string.bytes().len();
            let res_str: string_t = res_string.to_string_t();

            unsafe { res_ptr.write(res_str) }

            Ok(len as c_int)
        })
    }
}

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
