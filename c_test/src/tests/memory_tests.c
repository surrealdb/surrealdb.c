#include "surrealdb.h"
#include "unity_fixture.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Memory);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Memory) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Memory) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Memory, FreeArr) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t *results;
    int len = sr_select(db, &err, &results, "nonexistent_table");
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "select should succeed");

    if (len > 0) {
        sr_free_arr(results, len);
    }
    // If we get here without crashing, test passes
}

TEST(Memory, FreeBytes) {
    // Create a bytes structure manually for testing
    // sr_bytes_t is typically returned from RPC operations
    // For now, test with an empty/zero bytes struct
    sr_bytes_t bytes = {0};
    bytes.arr = NULL;
    bytes.len = 0;

    // This should handle NULL/empty data gracefully
    sr_free_bytes(bytes);
    // Test passes if we get here without crashing
}

TEST(Memory, FreeByteArr) {
    // Allocate a small byte array to test freeing
    // Note: sr_free_byte_arr expects memory allocated by Rust
    // We test with NULL to verify it handles edge cases
    sr_free_byte_arr(NULL, 0);
    // Test passes if we get here without crashing
}

TEST(Memory, FreeObject) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "key", "value");
    sr_free_object(obj);
    // If we get here without crashing, test passes
}

TEST(Memory, FreeArrRes) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    // Run a simple query to get arr_res data
    sr_arr_res_t *results;
    int len = sr_query(db, &err, &results, "RETURN 1", NULL);

    if (len < 0) {
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE("Query should succeed");
    }

    // Free individual result if we have one
    if (len > 0) {
        sr_free_arr_res(results[0]);
        // Note: We already freed the first element, so we can't use sr_free_arr_res_arr
        // Just free the array pointer itself
        // Actually, let's test differently - run another query
    }

    // Run another query to test sr_free_arr_res_arr
    len = sr_query(db, &err, &results, "RETURN [1, 2, 3]", NULL);
    if (len > 0) {
        sr_free_arr_res_arr(results, len);
    }
    // Test passes if we get here without crashing
}

TEST(Memory, FreeArrResArr) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_arr_res_t *results;
    int len = sr_query(db, &err, &results, "SELECT * FROM test", NULL);
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "query should succeed");

    if (len > 0) {
        sr_free_arr_res_arr(results, len);
    }
    // If we get here without crashing, test passes
}

TEST(Memory, FreeString) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_string_t version;
    sr_version(db, &err, &version);
    sr_free_string(version);
    // If we get here without crashing, test passes
}

TEST_GROUP_RUNNER(Memory) {
    RUN_TEST_CASE(Memory, FreeArr);
    RUN_TEST_CASE(Memory, FreeBytes);
    RUN_TEST_CASE(Memory, FreeByteArr);
    RUN_TEST_CASE(Memory, FreeObject);
    RUN_TEST_CASE(Memory, FreeArrRes);
    RUN_TEST_CASE(Memory, FreeArrResArr);
    RUN_TEST_CASE(Memory, FreeString);
}
