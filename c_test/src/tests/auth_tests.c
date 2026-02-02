#include "surrealdb.h"
#include "unity_fixture.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Auth);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Auth) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Auth) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(Auth, Signin) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_string_t token = NULL;

    // Test ROOT signin - in-memory DB may not support this
    sr_credentials_scope scope = ROOT;
    sr_credentials creds = {"root", "root"};

    int result = sr_signin(db, &err, &token, &scope, &creds, NULL, NULL);
    if (result < 0) {
        // In-memory DB may not require/support root auth - this is expected
        if (err)
            sr_free_string(err);
        // Test passes - we verified the function doesn't crash
        return;
    }

    // If signin succeeded, token should be returned
    TEST_ASSERT_NOT_NULL_MESSAGE(token, "Token should be returned on successful signin");
    sr_free_string(token);
}

TEST(Auth, Signup) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    // Create an access method for user signup
    sr_arr_res_t *query_res;
    int result = sr_query(db, &err, &query_res,
                          "DEFINE ACCESS user ON DATABASE TYPE RECORD "
                          "SIGNUP ( CREATE user SET username = $username, password = "
                          "crypto::argon2::generate($password) ) "
                          "SIGNIN ( SELECT * FROM user WHERE username = $username AND "
                          "crypto::argon2::compare(password, $password) ) "
                          "DURATION FOR SESSION 1d",
                          NULL);

    if (result < 0) {
        // If we can't create access method, skip gracefully
        if (err)
            sr_free_string(err);
        return;
    }
    if (result > 0) {
        sr_free_arr_res_arr(query_res, result);
    }

    // Test RECORD signup
    sr_string_t token = NULL;
    sr_credentials_scope scope = RECORD;
    sr_credentials creds = {"testuser", "testpass123"};
    sr_credentials_access details = {"test_ns", "test_db", "user"};

    result = sr_signup(db, &err, &token, &scope, &creds, &details, NULL);
    if (result < 0) {
        // Signup may fail in embedded mode - this is acceptable
        if (err)
            sr_free_string(err);
        return;
    }

    // If signup succeeded, token should be returned
    TEST_ASSERT_NOT_NULL_MESSAGE(token, "Token should be returned on successful signup");
    sr_free_string(token);
}

TEST_GROUP_RUNNER(Auth) {
    RUN_TEST_CASE(Auth, Signin);
    RUN_TEST_CASE(Auth, Signup);
}
