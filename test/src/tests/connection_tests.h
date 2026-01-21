#ifndef CONNECTION_TESTS_H
#define CONNECTION_TESTS_H

#include <stdio.h>
#include <string.h>
#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_connect(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_disconnect(db);
}

static inline void test_sr_surreal_disconnect(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_disconnect(db);
}

static inline void test_sr_authenticate(void) {
    TEST_IGNORE_MESSAGE("Authentication test not yet implemented");
}

static inline void test_sr_invalidate(void) {
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

static inline void test_sr_use_ns(void) {
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

static inline void test_sr_use_db(void) {
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

static inline void test_sr_version(void) {
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

static inline void test_sr_health(void) {
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

#endif // CONNECTION_TESTS_H
