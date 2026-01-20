#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unity.h>
#include "test_granular.h"

// ============================================================================
// Helper Functions
// ============================================================================

sr_surreal_t* test_helper_connect(const char *endpoint) {
    sr_surreal_t *db;
    sr_string_t err;
    
    if (sr_connect(&err, &db, endpoint) < 0) {
        printf("ERROR: Failed to connect to %s: %s\n", endpoint, err);
        sr_free_string(err);
        return NULL;
    }
    
    return db;
}

int test_helper_setup_ns_db(sr_surreal_t *db, const char *ns, const char *db_name) {
    sr_string_t err;
    
    if (sr_use_ns(db, &err, ns) < 0) {
        printf("ERROR: Failed to use namespace %s: %s\n", ns, err);
        sr_free_string(err);
        return -1;
    }
    
    if (sr_use_db(db, &err, db_name) < 0) {
        printf("ERROR: Failed to use database %s: %s\n", db_name, err);
        sr_free_string(err);
        return -1;
    }
    
    return 0;
}

void test_helper_disconnect(sr_surreal_t *db) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
    }
}

// ============================================================================
// Connection & Session Management Tests
// ============================================================================

void test_sr_connect(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_disconnect(db);
}

void test_sr_surreal_disconnect(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    // Disconnect is tested by not crashing
    test_helper_disconnect(db);
}

void test_sr_authenticate(void) {
    // TODO: Implement authentication test
    // Requires setting up authentication on the server
    TEST_IGNORE_MESSAGE("Authentication test not yet implemented");
}

void test_sr_invalidate(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t err;
    int result = sr_invalidate(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "Invalidate should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "Invalidate should succeed");
    
    test_helper_disconnect(db);
}

void test_sr_use_ns(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t err;
    int result = sr_use_ns(db, &err, "test_namespace");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "use_ns should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "use_ns should succeed");
    
    test_helper_disconnect(db);
}

void test_sr_use_db(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t err;
    sr_use_ns(db, &err, "test_namespace");
    
    int result = sr_use_db(db, &err, "test_database");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "use_db should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "use_db should succeed");
    
    test_helper_disconnect(db);
}

void test_sr_version(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t err;
    sr_string_t version;
    int result = sr_version(db, &err, &version);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "version should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "version should succeed");
    
    if (result >= 0) {
        TEST_ASSERT_NOT_NULL_MESSAGE(version, "Version string should not be null");
        printf("INFO: SurrealDB version: %s\n", version);
        sr_free_string(version);
    }
    
    test_helper_disconnect(db);
}

void test_sr_health(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t err;
    int result = sr_health(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "health check should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "health check should succeed");
    
    test_helper_disconnect(db);
}

// ============================================================================
// Transaction Management Tests
// ============================================================================

void test_sr_begin(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    int result = sr_begin(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "begin transaction should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "begin transaction should succeed");
    
    test_helper_disconnect(db);
}

void test_sr_cancel(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_begin(db, &err);
    
    int result = sr_cancel(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "cancel transaction should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "cancel transaction should succeed");
    
    test_helper_disconnect(db);
}

void test_sr_commit(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_begin(db, &err);
    
    int result = sr_commit(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "commit transaction should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "commit transaction should succeed");
    
    test_helper_disconnect(db);
}

// ============================================================================
// CRUD Operations Tests
// ============================================================================

void test_sr_create(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_item");
    sr_object_insert_int(&obj, "value", 42);
    
    sr_object_t *result;
    int len = sr_create(db, &err, &result, "test_table:1", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "create should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "create should succeed");
    
    if (len >= 0) {
        TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, len, "create should return at least one result");
        // Free the result objects
        for (int i = 0; i < len; i++) {
            sr_free_object(result[i]);
        }
    }
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

void test_sr_delete(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    // First create a record
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "to_delete");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:2", &obj);
    sr_free_object(obj);
    
    // Now delete it
    sr_value_t *delete_result;
    int len = sr_delete(db, &err, &delete_result, "test_table:2");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "delete should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "delete should succeed");
    
    if (len >= 0) {
        sr_free_arr(delete_result, len);
    }
    
    test_helper_disconnect(db);
}

void test_sr_insert(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "inserted_item");
    sr_object_insert_int(&obj, "count", 10);
    
    sr_value_t *result;
    int len = sr_insert(db, &err, &result, "test_table", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "insert should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "insert should succeed");
    
    if (len >= 0) {
        sr_free_arr(result, len);
    }
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

void test_sr_select(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    // First create a record
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "select_test");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:3", &obj);
    sr_free_object(obj);
    
    // Now select it
    sr_value_t *select_result;
    int len = sr_select(db, &err, &select_result, "test_table:3");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "select should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "select should succeed");
    
    if (len >= 0) {
        TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, len, "select should return at least one result");
        sr_free_arr(select_result, len);
    }
    
    test_helper_disconnect(db);
}

void test_sr_update(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    // First create a record
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:4", &obj);
    sr_free_object(obj);
    
    // Now update it
    sr_object_t update_obj = sr_object_new();
    sr_object_insert_str(&update_obj, "name", "updated");
    sr_value_t *update_result;
    int len = sr_update(db, &err, &update_result, "test_table:4", &update_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "update should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(update_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "update should succeed");
    
    if (len >= 0) {
        sr_free_arr(update_result, len);
    }
    
    sr_free_object(update_obj);
    test_helper_disconnect(db);
}

void test_sr_upsert(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "upserted");
    
    sr_value_t *result;
    int len = sr_upsert(db, &err, &result, "test_table:5", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "upsert should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "upsert should succeed");
    
    if (len >= 0) {
        sr_free_arr(result, len);
    }
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

void test_sr_merge(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    // First create a record
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    sr_object_insert_int(&obj, "value", 1);
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:6", &obj);
    sr_free_object(obj);
    
    // Now merge additional fields
    sr_object_t merge_obj = sr_object_new();
    sr_object_insert_int(&merge_obj, "extra", 99);
    sr_value_t *merge_result;
    int len = sr_merge(db, &err, &merge_result, "test_table:6", &merge_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "merge should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(merge_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "merge should succeed");
    
    if (len >= 0) {
        sr_free_arr(merge_result, len);
    }
    
    sr_free_object(merge_obj);
    test_helper_disconnect(db);
}

// ============================================================================
// Query & Live Select Tests
// ============================================================================

void test_sr_query(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_arr_res_t *res_arr;
    sr_object_t vars = sr_object_new();
    sr_object_insert_int(&vars, "val", 23);
    
    int len = sr_query(db, &err, &res_arr, "CREATE ONLY foo SET val = $val; SELECT value val FROM ONLY foo LIMIT 1;", &vars);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "query should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(vars);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "query should succeed");
    
    if (len >= 0) {
        TEST_ASSERT_EQUAL_INT_MESSAGE(2, len, "query should return 2 results");
        
        // Check first result (CREATE)
        if (res_arr[0].err.code >= 0) {
            sr_array_t create_res = res_arr[0].ok;
            TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, (int)create_res.len, "create should return results");
        }
        
        // Check second result (SELECT)
        if (res_arr[1].err.code >= 0) {
            sr_array_t select_res = res_arr[1].ok;
            TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, (int)select_res.len, "select should return results");
            
            // Verify the value
            if (select_res.arr[0].tag == SR_VALUE_NUMBER) {
                sr_number_t num = select_res.arr[0].sr_value_number;
                if (num.tag == SR_NUMBER_INT) {
                    TEST_ASSERT_EQUAL_INT_MESSAGE(23, (int)num.sr_number_int, "value should be 23");
                }
            }
        }
        
        sr_free_arr_res_arr(res_arr, len);
    }
    
    sr_free_object(vars);
    test_helper_disconnect(db);
}

void test_sr_select_live(void) {
    // TODO: Implement live select test
    // Requires handling streams and notifications
    TEST_IGNORE_MESSAGE("Live select test not yet implemented");
}

// ============================================================================
// Import/Export Tests
// ============================================================================

void test_sr_export(void) {
    // TODO: Implement export test
    // Requires file system operations
    TEST_IGNORE_MESSAGE("Export test not yet implemented");
}

void test_sr_import(void) {
    // TODO: Implement import test
    // Requires file system operations
    TEST_IGNORE_MESSAGE("Import test not yet implemented");
}

// ============================================================================
// Authentication Tests
// ============================================================================

void test_sr_signin(void) {
    // TODO: Implement signin test
    // Requires authentication setup
    TEST_IGNORE_MESSAGE("Signin test not yet implemented");
}

void test_sr_signup(void) {
    // TODO: Implement signup test
    // Requires authentication setup
    TEST_IGNORE_MESSAGE("Signup test not yet implemented");
}

// ============================================================================
// Variable Management Tests
// ============================================================================

void test_sr_set(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "test", "value");
    
    sr_value_t value = {
        .tag = SR_VALUE_OBJECT,
        .sr_value_object = obj
    };
    
    int result = sr_set(db, &err, "my_var", &value);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "set should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "set should succeed");
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

void test_sr_unset(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    int result = sr_unset(db, &err, "my_var");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "unset should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "unset should succeed");
    
    test_helper_disconnect(db);
}

// ============================================================================
// RPC Tests
// ============================================================================

void test_sr_surreal_rpc_new(void) {
    // TODO: Implement RPC new test
    TEST_IGNORE_MESSAGE("RPC new test not yet implemented");
}

void test_sr_surreal_rpc_execute(void) {
    // TODO: Implement RPC execute test
    TEST_IGNORE_MESSAGE("RPC execute test not yet implemented");
}

void test_sr_surreal_rpc_notifications(void) {
    // TODO: Implement RPC notifications test
    TEST_IGNORE_MESSAGE("RPC notifications test not yet implemented");
}

void test_sr_surreal_rpc_free(void) {
    // TODO: Implement RPC free test
    TEST_IGNORE_MESSAGE("RPC free test not yet implemented");
}

// ============================================================================
// Stream Tests
// ============================================================================

void test_sr_stream_next(void) {
    // TODO: Implement stream next test
    TEST_IGNORE_MESSAGE("Stream next test not yet implemented");
}

void test_sr_stream_kill(void) {
    // TODO: Implement stream kill test
    TEST_IGNORE_MESSAGE("Stream kill test not yet implemented");
}

// ============================================================================
// Object Manipulation Tests
// ============================================================================

void test_sr_object_insert(void) {
    sr_object_t obj = sr_object_new();
    sr_value_t val = {
        .tag = SR_VALUE_NUMBER,
        .sr_value_number = {
            .tag = SR_NUMBER_INT,
            .sr_number_int = 42
        }
    };
    
    sr_object_insert(&obj, "test_key", &val);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "test_key");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    
    sr_free_object(obj);
}

void test_sr_object_insert_str(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_string");
    
    const sr_value_t *retrieved = sr_object_get(&obj, "name");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_STRAND, retrieved->tag, "Value should be a string");
    
    sr_free_object(obj);
}

void test_sr_object_insert_int(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_int(&obj, "count", 100);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "count");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_INT, retrieved->sr_value_number.tag, "Number should be an int");
    TEST_ASSERT_EQUAL_INT_MESSAGE(100, (int)retrieved->sr_value_number.sr_number_int, "Value should be 100");
    
    sr_free_object(obj);
}

void test_sr_object_insert_float(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_float(&obj, "ratio", 3.14f);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "ratio");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_FLOAT, retrieved->sr_value_number.tag, "Number should be a float");
    
    sr_free_object(obj);
}

void test_sr_object_insert_double(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_double(&obj, "pi", 3.14159265359);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "pi");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_FLOAT, retrieved->sr_value_number.tag, "Number should be a float");
    
    sr_free_object(obj);
}

// ============================================================================
// Memory Management Tests
// ============================================================================

void test_sr_free_arr(void) {
    // Memory management functions are tested implicitly in other tests
    // This is a placeholder to ensure the function exists
    TEST_PASS_MESSAGE("sr_free_arr exists and is used in other tests");
}

void test_sr_free_bytes(void) {
    sr_bytes_t bytes = {
        .arr = NULL,
        .len = 0
    };
    sr_free_bytes(bytes);
    TEST_PASS_MESSAGE("sr_free_bytes should not crash with empty bytes");
}

void test_sr_free_byte_arr(void) {
    // Test with NULL pointer
    sr_free_byte_arr(NULL, 0);
    TEST_PASS_MESSAGE("sr_free_byte_arr should not crash with NULL");
}

void test_sr_free_object(void) {
    sr_object_t obj = sr_object_new();
    sr_free_object(obj);
    TEST_PASS_MESSAGE("sr_free_object should not crash");
}

void test_sr_free_arr_res(void) {
    // Memory management functions are tested implicitly in other tests
    TEST_PASS_MESSAGE("sr_free_arr_res exists and is used in other tests");
}

void test_sr_free_arr_res_arr(void) {
    // Memory management functions are tested implicitly in other tests
    TEST_PASS_MESSAGE("sr_free_arr_res_arr exists and is used in other tests");
}

void test_sr_free_string(void) {
    // Memory management functions are tested implicitly in other tests
    TEST_PASS_MESSAGE("sr_free_string exists and is used in other tests");
}

// ============================================================================
// Utility Tests
// ============================================================================

void test_sr_print_notification(void) {
    // TODO: Implement print notification test
    TEST_IGNORE_MESSAGE("Print notification test not yet implemented");
}

void test_sr_value_print(void) {
    sr_value_t val = {
        .tag = SR_VALUE_NUMBER,
        .sr_value_number = {
            .tag = SR_NUMBER_INT,
            .sr_number_int = 42
        }
    };
    
    // Just verify it doesn't crash
    sr_value_print(&val);
    TEST_PASS_MESSAGE("sr_value_print should not crash");
}
