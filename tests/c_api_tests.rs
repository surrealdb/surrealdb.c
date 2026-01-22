//! Integration tests that call C test functions to verify the SurrealDB C API.
//!
//! This module compiles and links against the C test library located in
//! `test/src/api_tests/` and calls each test function from Rust.

// Ensure we link against the surrealdb_c crate (provides the sr_* symbols)
extern crate surrealdb_c;

use std::ffi::c_int;

// Link to the C test library (compiled by build.rs)
#[link(name = "api_tests", kind = "static")]
extern "C" {
    // Connection Tests
    fn test_sr_connect() -> c_int;
    fn test_sr_surreal_disconnect() -> c_int;
    fn test_sr_use_ns() -> c_int;
    fn test_sr_use_db() -> c_int;
    fn test_sr_version() -> c_int;
    fn test_sr_health() -> c_int;

    // Authentication Tests
    fn test_sr_authenticate() -> c_int;
    fn test_sr_signin() -> c_int;
    fn test_sr_signup() -> c_int;
    fn test_sr_invalidate() -> c_int;

    // CRUD Tests
    fn test_sr_create() -> c_int;
    fn test_sr_select() -> c_int;
    fn test_sr_insert() -> c_int;
    fn test_sr_insert_relation() -> c_int;
    fn test_sr_update() -> c_int;
    fn test_sr_upsert() -> c_int;
    fn test_sr_delete() -> c_int;
    fn test_sr_merge() -> c_int;

    // Query Tests
    fn test_sr_query() -> c_int;
    fn test_sr_run() -> c_int;
    fn test_sr_relate() -> c_int;

    // Patch Tests
    fn test_sr_patch_add() -> c_int;
    fn test_sr_patch_remove() -> c_int;
    fn test_sr_patch_replace() -> c_int;

    // Transaction Tests
    fn test_sr_begin() -> c_int;
    fn test_sr_commit() -> c_int;
    fn test_sr_cancel() -> c_int;

    // Session Variable Tests
    fn test_sr_set() -> c_int;
    fn test_sr_unset() -> c_int;

    // Live Query Tests
    fn test_sr_select_live() -> c_int;

    // Import/Export Tests
    fn test_sr_export() -> c_int;
    fn test_sr_import() -> c_int;

    // Value Creation Tests
    fn test_sr_value_none() -> c_int;
    fn test_sr_value_null() -> c_int;
    fn test_sr_value_bool() -> c_int;
    fn test_sr_value_int() -> c_int;
    fn test_sr_value_float() -> c_int;
    fn test_sr_value_string() -> c_int;
    fn test_sr_value_object() -> c_int;
    fn test_sr_value_free() -> c_int;
    fn test_sr_value_duration() -> c_int;
    fn test_sr_value_datetime() -> c_int;
    fn test_sr_value_uuid() -> c_int;
    fn test_sr_value_array() -> c_int;
    fn test_sr_value_bytes() -> c_int;
    fn test_sr_value_thing() -> c_int;
    fn test_sr_value_point() -> c_int;
    fn test_sr_value_linestring() -> c_int;
    fn test_sr_value_polygon() -> c_int;
    fn test_sr_value_multipoint() -> c_int;

    // Object Manipulation Tests
    fn test_sr_object_new() -> c_int;
    fn test_sr_object_get() -> c_int;
    fn test_sr_object_insert() -> c_int;
    fn test_sr_object_insert_str() -> c_int;
    fn test_sr_object_insert_int() -> c_int;
    fn test_sr_object_insert_float() -> c_int;
    fn test_sr_object_insert_double() -> c_int;
    fn test_sr_free_object() -> c_int;

    // Array Tests
    fn test_sr_free_arr() -> c_int;

    // RPC Tests
    fn test_sr_surreal_rpc_new() -> c_int;
    fn test_sr_surreal_rpc_execute() -> c_int;
    fn test_sr_surreal_rpc_notifications() -> c_int;
    fn test_sr_surreal_rpc_free() -> c_int;

    // Stream Tests
    fn test_sr_stream_next() -> c_int;
    fn test_sr_stream_kill() -> c_int;
    fn test_sr_rpc_stream_next() -> c_int;
    fn test_sr_rpc_stream_free() -> c_int;

    // Utility Tests
    fn test_sr_free_string() -> c_int;
    fn test_sr_value_print() -> c_int;
    fn test_sr_value_eq() -> c_int;
    fn test_sr_print_notification() -> c_int;
}

const TEST_PASS: c_int = 0;
const TEST_FAIL: c_int = 1;
const TEST_SKIP: c_int = 2;

/// Helper macro to run a C test function and handle the result
macro_rules! c_test {
    ($rust_name:ident, $c_func:ident) => {
        #[test]
        fn $rust_name() {
            let result = unsafe { $c_func() };
            match result {
                TEST_PASS => {}
                TEST_SKIP => {
                    println!("Test skipped");
                }
                TEST_FAIL => panic!("C test failed"),
                code => panic!("C test returned unexpected code: {}", code),
            }
        }
    };
}

// ============================================================================
// Connection Tests
// ============================================================================

c_test!(sr_connect, test_sr_connect);
c_test!(sr_surreal_disconnect, test_sr_surreal_disconnect);
c_test!(sr_use_ns, test_sr_use_ns);
c_test!(sr_use_db, test_sr_use_db);
c_test!(sr_version, test_sr_version);
c_test!(sr_health, test_sr_health);

// ============================================================================
// Authentication Tests
// ============================================================================

c_test!(sr_authenticate, test_sr_authenticate);
c_test!(sr_signin, test_sr_signin);
c_test!(sr_signup, test_sr_signup);
c_test!(sr_invalidate, test_sr_invalidate);

// ============================================================================
// CRUD Tests
// ============================================================================

c_test!(sr_create, test_sr_create);
c_test!(sr_select, test_sr_select);
c_test!(sr_insert, test_sr_insert);
c_test!(sr_insert_relation, test_sr_insert_relation);
c_test!(sr_update, test_sr_update);
c_test!(sr_upsert, test_sr_upsert);
c_test!(sr_delete, test_sr_delete);
c_test!(sr_merge, test_sr_merge);

// ============================================================================
// Query Tests
// ============================================================================

c_test!(sr_query, test_sr_query);
c_test!(sr_run, test_sr_run);
c_test!(sr_relate, test_sr_relate);

// ============================================================================
// Patch Tests
// ============================================================================

c_test!(sr_patch_add, test_sr_patch_add);
c_test!(sr_patch_remove, test_sr_patch_remove);
c_test!(sr_patch_replace, test_sr_patch_replace);

// ============================================================================
// Transaction Tests
// ============================================================================

c_test!(sr_begin, test_sr_begin);
c_test!(sr_commit, test_sr_commit);
c_test!(sr_cancel, test_sr_cancel);

// ============================================================================
// Session Variable Tests
// ============================================================================

c_test!(sr_set, test_sr_set);
c_test!(sr_unset, test_sr_unset);

// ============================================================================
// Live Query Tests
// ============================================================================

c_test!(sr_select_live, test_sr_select_live);

// ============================================================================
// Import/Export Tests
// ============================================================================

c_test!(sr_export, test_sr_export);
c_test!(sr_import, test_sr_import);

// ============================================================================
// Value Creation Tests
// ============================================================================

c_test!(sr_value_none, test_sr_value_none);
c_test!(sr_value_null, test_sr_value_null);
c_test!(sr_value_bool, test_sr_value_bool);
c_test!(sr_value_int, test_sr_value_int);
c_test!(sr_value_float, test_sr_value_float);
c_test!(sr_value_string, test_sr_value_string);
c_test!(sr_value_object, test_sr_value_object);
c_test!(sr_value_free, test_sr_value_free);
c_test!(sr_value_duration, test_sr_value_duration);
c_test!(sr_value_datetime, test_sr_value_datetime);
c_test!(sr_value_uuid, test_sr_value_uuid);
c_test!(sr_value_array, test_sr_value_array);
c_test!(sr_value_bytes, test_sr_value_bytes);
c_test!(sr_value_thing, test_sr_value_thing);
c_test!(sr_value_point, test_sr_value_point);
c_test!(sr_value_linestring, test_sr_value_linestring);
c_test!(sr_value_polygon, test_sr_value_polygon);
c_test!(sr_value_multipoint, test_sr_value_multipoint);

// ============================================================================
// Object Manipulation Tests
// ============================================================================

c_test!(sr_object_new, test_sr_object_new);
c_test!(sr_object_get, test_sr_object_get);
c_test!(sr_object_insert, test_sr_object_insert);
c_test!(sr_object_insert_str, test_sr_object_insert_str);
c_test!(sr_object_insert_int, test_sr_object_insert_int);
c_test!(sr_object_insert_float, test_sr_object_insert_float);
c_test!(sr_object_insert_double, test_sr_object_insert_double);
c_test!(sr_free_object, test_sr_free_object);

// ============================================================================
// Array Tests
// ============================================================================

c_test!(sr_free_arr, test_sr_free_arr);

// ============================================================================
// RPC Tests
// ============================================================================

c_test!(sr_surreal_rpc_new, test_sr_surreal_rpc_new);
c_test!(sr_surreal_rpc_execute, test_sr_surreal_rpc_execute);
c_test!(sr_surreal_rpc_notifications, test_sr_surreal_rpc_notifications);
c_test!(sr_surreal_rpc_free, test_sr_surreal_rpc_free);

// ============================================================================
// Stream Tests
// ============================================================================

c_test!(sr_stream_next, test_sr_stream_next);
c_test!(sr_stream_kill, test_sr_stream_kill);
c_test!(sr_rpc_stream_next, test_sr_rpc_stream_next);
c_test!(sr_rpc_stream_free, test_sr_rpc_stream_free);

// ============================================================================
// Utility Tests
// ============================================================================

c_test!(sr_free_string, test_sr_free_string);
c_test!(sr_value_print, test_sr_value_print);
c_test!(sr_value_eq, test_sr_value_eq);
c_test!(sr_print_notification, test_sr_print_notification);
