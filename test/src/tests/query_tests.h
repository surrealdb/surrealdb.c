#ifndef QUERY_TESTS_H
#define QUERY_TESTS_H

#include <stdio.h>
#include <string.h>
#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_query(void) {
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
    
    if (len > 0) {
        sr_free_arr_res_arr(res_arr, len);
    }
    
    sr_free_object(vars);
    test_helper_disconnect(db);
}

static inline void test_sr_select_live(void) {
    TEST_IGNORE_MESSAGE("Live select test not yet implemented");
}

#endif // QUERY_TESTS_H
