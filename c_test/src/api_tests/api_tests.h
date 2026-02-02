/**
 * SurrealDB C API Test Library
 *
 * This library provides C test functions that can be called from Rust integration tests.
 * Each function tests a specific API function from the SurrealDB C bindings.
 *
 * Return values:
 *   0 = test passed
 *   non-zero = test failed (error code)
 */

#ifndef SURREALDB_API_TESTS_H
#define SURREALDB_API_TESTS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Test Result Codes
 * ============================================================================ */

#define TEST_PASS 0
#define TEST_FAIL 1
#define TEST_SKIP 2

/* ============================================================================
 * Connection Tests
 * ============================================================================ */

int test_sr_connect(void);
int test_sr_surreal_disconnect(void);
int test_sr_use_ns(void);
int test_sr_use_db(void);
int test_sr_version(void);
int test_sr_health(void);

/* ============================================================================
 * Authentication Tests
 * ============================================================================ */

int test_sr_authenticate(void);
int test_sr_signin(void);
int test_sr_signup(void);
int test_sr_invalidate(void);

/* ============================================================================
 * CRUD Tests
 * ============================================================================ */

int test_sr_create(void);
int test_sr_select(void);
int test_sr_insert(void);
int test_sr_insert_relation(void);
int test_sr_update(void);
int test_sr_upsert(void);
int test_sr_delete(void);
int test_sr_merge(void);

/* ============================================================================
 * Query Tests
 * ============================================================================ */

int test_sr_query(void);
int test_sr_run(void);
int test_sr_relate(void);

/* ============================================================================
 * Patch Tests
 * ============================================================================ */

int test_sr_patch_add(void);
int test_sr_patch_remove(void);
int test_sr_patch_replace(void);

/* ============================================================================
 * Transaction Tests
 * ============================================================================ */

int test_sr_begin(void);
int test_sr_commit(void);
int test_sr_cancel(void);

/* ============================================================================
 * Session Variable Tests
 * ============================================================================ */

int test_sr_set(void);
int test_sr_unset(void);

/* ============================================================================
 * Live Query Tests
 * ============================================================================ */

int test_sr_select_live(void);

/* ============================================================================
 * Import/Export Tests
 * ============================================================================ */

int test_sr_export(void);
int test_sr_import(void);

/* ============================================================================
 * Value Creation Tests
 * ============================================================================ */

int test_sr_value_none(void);
int test_sr_value_null(void);
int test_sr_value_bool(void);
int test_sr_value_int(void);
int test_sr_value_float(void);
int test_sr_value_string(void);
int test_sr_value_object(void);
int test_sr_value_free(void);
int test_sr_value_duration(void);
int test_sr_value_datetime(void);
int test_sr_value_uuid(void);
int test_sr_value_array(void);
int test_sr_value_bytes(void);
int test_sr_value_thing(void);
int test_sr_value_point(void);
int test_sr_value_linestring(void);
int test_sr_value_polygon(void);
int test_sr_value_multipoint(void);

/* ============================================================================
 * Object Manipulation Tests
 * ============================================================================ */

int test_sr_object_new(void);
int test_sr_object_get(void);
int test_sr_object_insert(void);
int test_sr_object_insert_str(void);
int test_sr_object_insert_int(void);
int test_sr_object_insert_float(void);
int test_sr_object_insert_double(void);
int test_sr_free_object(void);

/* ============================================================================
 * Array Tests
 * ============================================================================ */

int test_sr_free_arr(void);

/* ============================================================================
 * RPC Tests
 * ============================================================================ */

int test_sr_surreal_rpc_new(void);
int test_sr_surreal_rpc_execute(void);
int test_sr_surreal_rpc_notifications(void);
int test_sr_surreal_rpc_free(void);

/* ============================================================================
 * Stream Tests
 * ============================================================================ */

int test_sr_stream_next(void);
int test_sr_stream_kill(void);
int test_sr_rpc_stream_next(void);
int test_sr_rpc_stream_free(void);

/* ============================================================================
 * Utility Tests
 * ============================================================================ */

int test_sr_free_string(void);
int test_sr_value_print(void);
int test_sr_value_eq(void);
int test_sr_print_notification(void);

/* ============================================================================
 * Additional Geometry Tests
 * ============================================================================ */

int test_sr_value_multilinestring(void);
int test_sr_value_multipolygon(void);
int test_sr_value_decimal(void);

/* ============================================================================
 * Array Manipulation Tests
 * ============================================================================ */

int test_sr_array_len(void);
int test_sr_array_get(void);
int test_sr_array_push(void);

/* ============================================================================
 * Object Iteration Tests
 * ============================================================================ */

int test_sr_object_len(void);
int test_sr_object_keys(void);

/* ============================================================================
 * Kill Live Query Test
 * ============================================================================ */

int test_sr_kill(void);

#ifdef __cplusplus
}
#endif

#endif /* SURREALDB_API_TESTS_H */
