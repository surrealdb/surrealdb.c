#include "surrealdb.h"
#include "unity_fixture.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Transaction);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Transaction) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Transaction) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Transaction, Begin) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    int result = sr_begin(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "begin should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "begin should succeed");

    // Clean up - cancel the transaction
    sr_cancel(db, &err);
}

TEST(Transaction, Cancel) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_begin(db, &err);

    int result = sr_cancel(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "cancel should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "cancel should succeed");
}

TEST(Transaction, Commit) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_begin(db, &err);

    int result = sr_commit(db, &err);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "commit should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "commit should succeed");
}

TEST_GROUP_RUNNER(Transaction) {
    RUN_TEST_CASE(Transaction, Begin);
    RUN_TEST_CASE(Transaction, Cancel);
    RUN_TEST_CASE(Transaction, Commit);
}
