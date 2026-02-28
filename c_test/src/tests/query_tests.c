#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Query);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Query) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Query) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Query, Query) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_arr_res_t *results;
    int len = sr_query(db, &err, &results, "SELECT * FROM test_table", NULL);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "query should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "query should succeed");
    
    if (len > 0) {
        sr_free_arr_res_arr(results, len);
    }
}

TEST(Query, SelectLive) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    // In v3, table must exist before live select. Create it first.
    sr_arr_res_t *setup_results = NULL;
    int setup_len = sr_query(db, &err, &setup_results, "DEFINE TABLE test_table SCHEMALESS", NULL);
    if (setup_len > 0) sr_free_arr_res_arr(setup_results, setup_len);
    if (setup_len < 0 && err) { sr_free_string(err); err = NULL; }

    sr_stream_t *stream;
    int result = sr_select_live(db, &err, &stream, "test_table");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "select_live should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "select_live should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(stream, "Stream should not be NULL");
    
    sr_stream_kill(stream);
}

TEST_GROUP_RUNNER(Query) {
    RUN_TEST_CASE(Query, Query);
    RUN_TEST_CASE(Query, SelectLive);
}
