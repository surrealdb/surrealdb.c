#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Variable);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Variable) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Variable) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Variable, Set) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_value_t *value = sr_value_int(42);
    int result = sr_set(db, &err, "test_var", value);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "set should succeed: %s", err);
        sr_free_string(err);
        sr_value_free(value);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "set should succeed");
    
    sr_value_free(value);
}

TEST(Variable, Unset) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_value_t *value = sr_value_int(42);
    sr_set(db, &err, "test_var", value);
    sr_value_free(value);
    
    int result = sr_unset(db, &err, "test_var");
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "unset should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "unset should succeed");
}

TEST_GROUP_RUNNER(Variable) {
    RUN_TEST_CASE(Variable, Set);
    RUN_TEST_CASE(Variable, Unset);
}
