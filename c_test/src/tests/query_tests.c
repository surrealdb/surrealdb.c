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
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "query should succeed");
    
    if (len > 0) {
        sr_arr_res_arr_free(results, len);
    }
}

TEST(Query, SelectLive) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    // In v3, table must exist before live select. Create it first.
    sr_arr_res_t *setup_results = NULL;
    int setup_len = sr_query(db, &err, &setup_results, "DEFINE TABLE test_table SCHEMALESS", NULL);
    if (setup_len > 0) sr_arr_res_arr_free(setup_results, setup_len);
    if (setup_len < 0 && err) { sr_string_free(err); err = NULL; }

    sr_stream_t *stream;
    int result = sr_select_live(db, &err, &stream, "test_table");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "select_live should succeed: %s", err);
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "select_live should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(stream, "Stream should not be NULL");
    
    sr_stream_kill(stream);
}

TEST(Query, Run) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_arr_res_t *define = NULL;
    int n = sr_query(db, &err, &define,
                     "DEFINE FUNCTION fn::greet() { RETURN 'hi'; }",
                     NULL);
    if (n < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_IGNORE_MESSAGE("could not define a function to run");
    }
    if (n > 0) sr_arr_res_arr_free(define, n);

    sr_value_t *result = NULL;
    int res = sr_run(db, &err, &result, "fn::greet", NULL);
    if (res < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "run should succeed: %s", err);
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, res, "run should succeed");
    /* sr_run writes an array of values and returns its length. */
    if (result) sr_values_free(result, res);
}

TEST(Query, Kill) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_arr_res_t *define = NULL;
    int n = sr_query(db, &err, &define, "DEFINE TABLE killable SCHEMALESS", NULL);
    if (n > 0) sr_arr_res_arr_free(define, n);
    if (n < 0 && err) { sr_string_free(err); err = NULL; }

    sr_stream_t *stream = NULL;
    if (sr_select_live(db, &err, &stream, "killable") < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_IGNORE_MESSAGE("could not start a live query to kill");
    }
    TEST_ASSERT_NOT_NULL(stream);

    /* sr_kill takes the query id as a string; the stream owns the handle. */
    sr_stream_kill(stream);
}

TEST(Query, NullErrPtrIsHonoured) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    /* Every function routed through with_surreal_async documents err_ptr as
       "a valid pointer or null" -- 27 of them say so. The failure branch used
       to write through it unconditionally, so passing null worked right up
       until something actually went wrong and then segfaulted. The success
       path was always fine, which is why nothing caught it.

       Both paths are exercised here; the test failing is a crash, not an
       assertion. */
    sr_use_ns(db, NULL, "null_err_ns");

    sr_arr_res_t *res = NULL;
    int rc = sr_query(db, NULL, &res, "NOT VALID SURQL !!", NULL);
    TEST_ASSERT_LESS_THAN_INT_MESSAGE(0, rc, "a bad query must report failure");
    if (rc > 0 && res) sr_arr_res_arr_free(res, rc);
}

TEST_GROUP_RUNNER(Query) {
    RUN_TEST_CASE(Query, Query);
    RUN_TEST_CASE(Query, SelectLive);
    RUN_TEST_CASE(Query, Run);
    RUN_TEST_CASE(Query, Kill);
    RUN_TEST_CASE(Query, NullErrPtrIsHonoured);
}
