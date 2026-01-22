#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Connection);

static sr_surreal_t *db;

TEST_SETUP(Connection) {
    db = NULL;
}

TEST_TEAR_DOWN(Connection) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Connection, Connect) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "Connection should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Database handle should not be NULL");
}

TEST(Connection, Disconnect) {
    sr_string_t err;
    sr_connect(&err, &db, "memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    sr_surreal_disconnect(db);
    db = NULL;  // Mark as disconnected so teardown doesn't double-free
}

TEST(Connection, Authenticate) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    // In-memory databases don't require authentication,
    // but we can test that sr_authenticate accepts a token format
    // First invalidate any session
    result = sr_invalidate(db, &err);
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "Invalidate should succeed");
    
    // Try to authenticate with an invalid token - should fail gracefully
    result = sr_authenticate(db, &err, "invalid_token");
    // This should fail since the token is invalid, but shouldn't crash
    if (result < 0) {
        sr_free_string(err);
    }
    // Test passes if we get here without crashing
}

TEST(Connection, Invalidate) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    result = sr_invalidate(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "Invalidate should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "Invalidate should succeed");
}

TEST(Connection, UseNamespace) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    result = sr_use_ns(db, &err, "test_namespace");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "use_ns should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "use_ns should succeed");
}

TEST(Connection, UseDatabase) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    sr_use_ns(db, &err, "test_namespace");
    
    result = sr_use_db(db, &err, "test_database");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "use_db should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "use_db should succeed");
}

TEST(Connection, Version) {
    sr_string_t err;
    sr_string_t version;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    result = sr_version(db, &err, &version);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "version should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "version should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(version, "Version string should not be null");
    printf("  SurrealDB version: %s\n", version);
    sr_free_string(version);
}

TEST(Connection, Health) {
    sr_string_t err;
    int result = sr_connect(&err, &db, "memory");
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, result);
    
    result = sr_health(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "health check should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "health check should succeed");
}

TEST_GROUP_RUNNER(Connection) {
    RUN_TEST_CASE(Connection, Connect);
    RUN_TEST_CASE(Connection, Disconnect);
    RUN_TEST_CASE(Connection, Authenticate);
    RUN_TEST_CASE(Connection, Invalidate);
    RUN_TEST_CASE(Connection, UseNamespace);
    RUN_TEST_CASE(Connection, UseDatabase);
    RUN_TEST_CASE(Connection, Version);
    RUN_TEST_CASE(Connection, Health);
}
