#ifndef VARIABLE_TESTS_H
#define VARIABLE_TESTS_H

#include <stdio.h>
#include <string.h>
#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_set(void) {
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

static inline void test_sr_unset(void) {
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

#endif // VARIABLE_TESTS_H
