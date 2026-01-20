#ifndef TEST_GRANULAR_H
#define TEST_GRANULAR_H

#include "../../include/surrealdb.h"

// ============================================================================
// Connection & Session Management Tests
// ============================================================================

void test_sr_connect(void);
void test_sr_surreal_disconnect(void);
void test_sr_authenticate(void);
void test_sr_invalidate(void);
void test_sr_use_ns(void);
void test_sr_use_db(void);
void test_sr_version(void);
void test_sr_health(void);

// ============================================================================
// Transaction Management Tests
// ============================================================================

void test_sr_begin(void);
void test_sr_cancel(void);
void test_sr_commit(void);

// ============================================================================
// CRUD Operations Tests
// ============================================================================

void test_sr_create(void);
void test_sr_delete(void);
void test_sr_insert(void);
void test_sr_select(void);
void test_sr_update(void);
void test_sr_upsert(void);
void test_sr_merge(void);

// ============================================================================
// Query & Live Select Tests
// ============================================================================

void test_sr_query(void);
void test_sr_select_live(void);

// ============================================================================
// Import/Export Tests
// ============================================================================

void test_sr_export(void);
void test_sr_import(void);

// ============================================================================
// Authentication Tests
// ============================================================================

void test_sr_signin(void);
void test_sr_signup(void);

// ============================================================================
// Variable Management Tests
// ============================================================================

void test_sr_set(void);
void test_sr_unset(void);

// ============================================================================
// RPC Tests
// ============================================================================

void test_sr_surreal_rpc_new(void);
void test_sr_surreal_rpc_execute(void);
void test_sr_surreal_rpc_notifications(void);
void test_sr_surreal_rpc_free(void);

// ============================================================================
// Stream Tests
// ============================================================================

void test_sr_stream_next(void);
void test_sr_stream_kill(void);

// ============================================================================
// Object Manipulation Tests
// ============================================================================

void test_sr_object_insert(void);
void test_sr_object_insert_str(void);
void test_sr_object_insert_int(void);
void test_sr_object_insert_float(void);
void test_sr_object_insert_double(void);

// ============================================================================
// Memory Management Tests
// ============================================================================

void test_sr_free_arr(void);
void test_sr_free_bytes(void);
void test_sr_free_byte_arr(void);
void test_sr_free_object(void);
void test_sr_free_arr_res(void);
void test_sr_free_arr_res_arr(void);
void test_sr_free_string(void);

// ============================================================================
// Utility Tests
// ============================================================================

void test_sr_print_notification(void);
void test_sr_value_print(void);

// ============================================================================
// Helper Functions
// ============================================================================

// Helper to create a test database connection
sr_surreal_t* test_helper_connect(const char *endpoint);

// Helper to setup namespace and database
int test_helper_setup_ns_db(sr_surreal_t *db, const char *ns, const char *db_name);

// Helper to cleanup connection
void test_helper_disconnect(sr_surreal_t *db);

#endif // TEST_GRANULAR_H
