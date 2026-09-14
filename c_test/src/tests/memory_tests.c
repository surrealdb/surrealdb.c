#include "unity_fixture.h"
#include "surrealdb.h"
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
    
    // In v3, selecting from a nonexistent table may error.
    // Create the table first to ensure a clean result.
    sr_arr_res_t *setup_results = NULL;
    int setup_len = sr_query(db, &err, &setup_results, "DEFINE TABLE memory_test SCHEMALESS", NULL);
    if (setup_len > 0) sr_arr_res_arr_free(setup_results, setup_len);
    if (setup_len < 0 && err) { sr_string_free(err); err = NULL; }

    sr_value_t *results;
    int len = sr_select(db, &err, &results, "memory_test");
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "select should succeed");
    
    if (len > 0) {
        sr_values_free(results, len);
    }
}

TEST(Memory, FreeByteArr) {
    // Allocate a small byte array to test freeing
    // Note: sr_byte_arr_free expects memory allocated by Rust
    // We test with NULL to verify it handles edge cases
    sr_byte_arr_free(NULL, 0);
    // Test passes if we get here without crashing
}

TEST(Memory, FreeObject) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "key", "value");
    sr_object_free(obj);
    // If we get here without crashing, test passes
}

TEST(Memory, FreeArrRes) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    // sr_query always hands back one allocation holding `len` results, and
    // sr_arr_res_arr_free releases the elements and that allocation together.
    // There is no way to release a single element: the spine is one block.
    //
    // This test used to free a single element and then abandon the array,
    // leaking the spine -- undetectable until the suite gained
    // -DSURREALDB_SANITIZE=address. The by-value sr_arr_res_free it used has
    // since been removed: nothing produced an sr_arr_res_t by value, so the
    // only way to call it was on a shallow copy of an element, which dropped
    // contents the array still owned.
    sr_arr_res_t *results = NULL;
    int len = sr_query(db, &err, &results, "RETURN 1", NULL);
    if (len < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("Query should succeed");
    }
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, len, "RETURN 1 should produce one result");
    sr_arr_res_arr_free(results, len);

    results = NULL;
    len = sr_query(db, &err, &results, "RETURN [1, 2, 3]", NULL);
    if (len < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("Query should succeed");
    }
    sr_arr_res_arr_free(results, len);
}

TEST(Memory, FreeArrResArr) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_arr_res_t *results;
    int len = sr_query(db, &err, &results, "SELECT * FROM test", NULL);
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "query should succeed");
    
    if (len > 0) {
        sr_arr_res_arr_free(results, len);
    }
    // If we get here without crashing, test passes
}

TEST(Memory, FreeString) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_string_t version;
    sr_version(db, &err, &version);
    sr_string_free(version);
    // If we get here without crashing, test passes
}

TEST(Memory, FreeCreatedObject) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "owned");

    /* With a non-null res_ptr the created record is written by value and is
       owned here. sr_create used to box it and hand back a pointer to the box;
       sr_object_free takes an Object by value, so the box itself was
       unreachable from C and leaked on every call. Nothing in this suite could
       observe that, which is why the build now offers -DSURREALDB_SANITIZE. */
    sr_object_t created;
    int rc = sr_create(db, &err, &created, "memory_created", &content);
    sr_object_free(content);

    if (rc < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("create should succeed");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(1, rc, "create with a result slot reports one record");
    TEST_ASSERT_NOT_NULL_MESSAGE(sr_object_get(&created, "name"),
                                 "the created record carries its content");
    sr_object_free(created);
}

TEST(Memory, DiscardCreatedObject) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    /* A null res_ptr is documented as discarding the result, and must not
       allocate anything the caller has no way to release. */
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "discarded");
    int rc = sr_create(db, &err, NULL, "memory_discarded", &content);
    sr_object_free(content);

    if (rc < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("create should succeed");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(0, rc, "discarding the result reports zero records");
}

TEST_GROUP_RUNNER(Memory) {
    RUN_TEST_CASE(Memory, FreeArr);
    RUN_TEST_CASE(Memory, FreeByteArr);
    RUN_TEST_CASE(Memory, FreeObject);
    RUN_TEST_CASE(Memory, FreeArrRes);
    RUN_TEST_CASE(Memory, FreeArrResArr);
    RUN_TEST_CASE(Memory, FreeString);
    RUN_TEST_CASE(Memory, FreeCreatedObject);
    RUN_TEST_CASE(Memory, DiscardCreatedObject);
}
