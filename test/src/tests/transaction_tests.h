#ifndef TRANSACTION_TESTS_H
#define TRANSACTION_TESTS_H

#include <stdio.h>
#include <string.h>
#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_begin(void) {
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

static inline void test_sr_cancel(void) {
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

static inline void test_sr_commit(void) {
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

#endif // TRANSACTION_TESTS_H
