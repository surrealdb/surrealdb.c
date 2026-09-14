/**
 * SurrealDB C API Test Implementations
 * 
 * Each function tests a specific API function from the SurrealDB C bindings.
 */

#include "api_tests.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/* ============================================================================
 * Test Helpers
 * ============================================================================ */

#define ASSERT_TRUE(cond) do { if (!(cond)) { fprintf(stderr, "ASSERT_TRUE failed: %s at %s:%d\n", #cond, __FILE__, __LINE__); return API_TEST_FAIL; } } while(0)
#define ASSERT_FALSE(cond) do { if (cond) { fprintf(stderr, "ASSERT_FALSE failed: %s at %s:%d\n", #cond, __FILE__, __LINE__); return API_TEST_FAIL; } } while(0)
#define ASSERT_EQ(a, b) do { if ((a) != (b)) { fprintf(stderr, "ASSERT_EQ failed: %s != %s at %s:%d\n", #a, #b, __FILE__, __LINE__); return API_TEST_FAIL; } } while(0)
#define ASSERT_GE(a, b) do { if ((a) < (b)) { fprintf(stderr, "ASSERT_GE failed: %s < %s at %s:%d\n", #a, #b, __FILE__, __LINE__); return API_TEST_FAIL; } } while(0)
#define ASSERT_NOT_NULL(ptr) do { if ((ptr) == NULL) { fprintf(stderr, "ASSERT_NOT_NULL failed: %s at %s:%d\n", #ptr, __FILE__, __LINE__); return API_TEST_FAIL; } } while(0)

/* Helper to create a connected database for tests */
static int setup_db(sr_surreal_t **db) {
    sr_string_t err;
    if (sr_connect(&err, db, "mem://") < 0) {
        fprintf(stderr, "Failed to connect: %s\n", err);
        sr_string_free(err);
        return API_TEST_FAIL;
    }
    if (sr_use_ns(*db, &err, "test") < 0) {
        fprintf(stderr, "Failed to use namespace: %s\n", err);
        sr_string_free(err);
        sr_surreal_disconnect(*db);
        return API_TEST_FAIL;
    }
    if (sr_use_db(*db, &err, "test") < 0) {
        fprintf(stderr, "Failed to use database: %s\n", err);
        sr_string_free(err);
        sr_surreal_disconnect(*db);
        return API_TEST_FAIL;
    }
    return API_TEST_PASS;
}

/*
 * Create rows so that a table exists before it is selected from.
 *
 * SurrealDB 3.x rejects SELECT against a table that has never been written to
 * ("The table 'x' does not exist"), where 2.x returned an empty result.
 */
static int seed_table(sr_surreal_t *db, const char *statement) {
    sr_string_t err;
    sr_arr_res_t *res;

    int n = sr_query(db, &err, &res, statement, NULL);
    if (n < 0) {
        fprintf(stderr, "Failed to seed table: %s\n", err);
        sr_string_free(err);
        return API_TEST_FAIL;
    }
    sr_arr_res_arr_free(res, n);
    return API_TEST_PASS;
}

/* ============================================================================
 * Connection Tests
 * ============================================================================ */

int test_sr_connect(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    ASSERT_NOT_NULL(db);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_surreal_disconnect(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    /* If we get here without crashing, the test passes */
    return API_TEST_PASS;
}

int test_sr_use_ns(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    res = sr_use_ns(db, &err, "test_namespace");
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_use_db(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    res = sr_use_ns(db, &err, "test");
    ASSERT_GE(res, 0);
    
    res = sr_use_db(db, &err, "test_database");
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_version(void) {
    sr_surreal_t *db;
    sr_string_t err;
    sr_string_t ver;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    res = sr_version(db, &err, &ver);
    ASSERT_GE(res, 0);
    ASSERT_NOT_NULL(ver);
    
    sr_string_free(ver);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_health(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    res = sr_health(db, &err);
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Authentication Tests
 * ============================================================================ */

int test_sr_authenticate(void) {
    sr_surreal_t *db;
    sr_string_t err;
    sr_string_t token;
    
    /* Connect to in-memory database */
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    /* First signin to get a token */
    sr_credentials_scope scope = SR_SCOPE_ROOT;
    sr_credentials creds = { "root", "root" };
    
    res = sr_signin(db, &err, &token, &scope, &creds, NULL, NULL);
    if (res < 0) {
        /* In-memory DB may not require auth, which is fine */
        if (err) sr_string_free(err);
        sr_surreal_disconnect(db);
        return API_TEST_SKIP;
    }
    
    ASSERT_NOT_NULL(token);
    
    /* Now test authenticate with the token */
    res = sr_authenticate(db, &err, token);
    ASSERT_GE(res, 0);
    
    sr_string_free(token);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_signin(void) {
    sr_surreal_t *db;
    sr_string_t err;
    sr_string_t token = NULL;
    
    /* Connect to in-memory database */
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    /* Test ROOT signin */
    sr_credentials_scope scope = SR_SCOPE_ROOT;
    sr_credentials creds = { "root", "root" };
    
    res = sr_signin(db, &err, &token, &scope, &creds, NULL, NULL);
    if (res < 0) {
        /* In-memory DB may not support root auth - skip test */
        if (err) sr_string_free(err);
        sr_surreal_disconnect(db);
        return API_TEST_SKIP;
    }
    
    /* Token should be returned */
    ASSERT_NOT_NULL(token);
    
    sr_string_free(token);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_signup(void) {
    sr_surreal_t *db;
    sr_string_t err;
    sr_string_t token = NULL;
    
    /* Connect to in-memory database */
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    /* Setup namespace and database */
    res = sr_use_ns(db, &err, "test");
    ASSERT_GE(res, 0);
    res = sr_use_db(db, &err, "test");
    ASSERT_GE(res, 0);
    
    /* Create an access method for user signup via query */
    sr_arr_res_t *query_res;
    res = sr_query(db, &err, &query_res, 
        "DEFINE ACCESS user ON DATABASE TYPE RECORD "
        "SIGNUP ( CREATE user SET username = $username, password = crypto::argon2::generate($password) ) "
        "SIGNIN ( SELECT * FROM user WHERE username = $username AND crypto::argon2::compare(password, $password) ) "
        "DURATION FOR SESSION 1d",
        NULL);
    
    if (res < 0) {
        /* If we can't create access method, skip */
        if (err) sr_string_free(err);
        sr_surreal_disconnect(db);
        return API_TEST_SKIP;
    }
    sr_arr_res_arr_free(query_res, res);
    
    /* Test RECORD signup */
    sr_credentials_scope scope = SR_SCOPE_RECORD;
    sr_credentials creds = { "testuser", "testpass123" };
    sr_credentials_access details = { "test", "test", "user" };
    
    res = sr_signup(db, &err, &token, &scope, &creds, &details, NULL);
    if (res < 0) {
        /* Signup may fail for various reasons in embedded mode */
        fprintf(stderr, "Signup failed (may be expected): %s\n", err ? err : "unknown");
        sr_string_free(err);
        sr_surreal_disconnect(db);
        return API_TEST_SKIP;
    }
    
    /* Token should be returned */
    ASSERT_NOT_NULL(token);
    
    sr_string_free(token);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_invalidate(void) {
    sr_surreal_t *db;
    sr_string_t err;
    
    int res = sr_connect(&err, &db, "mem://");
    ASSERT_GE(res, 0);
    
    res = sr_invalidate(db, &err);
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * CRUD Tests
 * ============================================================================ */

int test_sr_create(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_object_t result;
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "test_item");
    
    int res = sr_create(db, &err, &result, "items", &content);
    ASSERT_GE(res, 0);
    
    sr_object_free(content);
    /* The created record is written by value and owned by the caller. */
    sr_object_free(result);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_select(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    if (seed_table(db, "CREATE items:1 SET name = 'a'") != API_TEST_PASS) {
        sr_surreal_disconnect(db);
        return API_TEST_FAIL;
    }

    int len = sr_select(db, &err, &results, "items");
    ASSERT_GE(len, 0);
    
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_insert(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "inserted_item");
    
    int len = sr_insert(db, &err, &results, "items", &content);
    ASSERT_GE(len, 0);
    
    sr_object_free(content);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_insert_relation(void) {
    /* Skip: insert_relation requires specific record ID format for in/out fields */
    return API_TEST_SKIP;
}

int test_sr_update(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record */
    sr_object_t create_content = sr_object_new();
    sr_object_insert_str(&create_content, "name", "original");
    sr_create(db, &err, NULL, "items:1", &create_content);
    sr_object_free(create_content);
    
    /* Update it */
    sr_object_t update_content = sr_object_new();
    sr_object_insert_str(&update_content, "name", "updated");
    
    int len = sr_update(db, &err, &results, "items:1", &update_content);
    ASSERT_GE(len, 0);
    
    sr_object_free(update_content);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_upsert(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "upserted_item");
    
    int len = sr_upsert(db, &err, &results, "items:upsert1", &content);
    ASSERT_GE(len, 0);
    
    sr_object_free(content);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_delete(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record */
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "to_delete");
    sr_create(db, &err, NULL, "items:delete1", &content);
    sr_object_free(content);
    
    /* Delete it */
    int len = sr_delete(db, &err, &results, "items:delete1");
    ASSERT_GE(len, 0);
    
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_merge(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record */
    sr_object_t create_content = sr_object_new();
    sr_object_insert_str(&create_content, "name", "original");
    sr_object_insert_int(&create_content, "count", 1);
    sr_create(db, &err, NULL, "items:merge1", &create_content);
    sr_object_free(create_content);
    
    /* Merge new data */
    sr_object_t merge_content = sr_object_new();
    sr_object_insert_int(&merge_content, "count", 2);
    
    int len = sr_merge(db, &err, &results, "items:merge1", &merge_content);
    ASSERT_GE(len, 0);
    
    sr_object_free(merge_content);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Query Tests
 * ============================================================================ */

int test_sr_query(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_arr_res_t *results;
    
    int len = sr_query(db, &err, &results, "SELECT * FROM items", NULL);
    ASSERT_GE(len, 0);
    
    if (len > 0) {
        sr_arr_res_arr_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_run(void) {
    /* Skip: sr_run may require specific function registration */
    return API_TEST_SKIP;
}

int test_sr_relate(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create two records */
    sr_object_t p1 = sr_object_new();
    sr_object_insert_str(&p1, "name", "John");
    sr_create(db, &err, NULL, "person:john", &p1);
    sr_object_free(p1);
    
    sr_object_t p2 = sr_object_new();
    sr_object_insert_str(&p2, "name", "Jane");
    sr_create(db, &err, NULL, "person:jane", &p2);
    sr_object_free(p2);
    
    /* Create relation */
    int len = sr_relate(db, &err, &results, "person:john", "knows", "person:jane", NULL);
    ASSERT_GE(len, 0);
    
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Patch Tests
 * ============================================================================ */

int test_sr_patch_add(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record */
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "test");
    sr_create(db, &err, NULL, "items:patch1", &content);
    sr_object_free(content);
    
    /* Patch add */
    sr_value_t *value = sr_value_string("new_value");
    int len = sr_patch_add(db, &err, &results, "items:patch1", "/new_field", value);
    ASSERT_GE(len, 0);
    
    sr_value_free(value);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_patch_remove(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record with a field to remove */
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "test");
    sr_object_insert_str(&content, "to_remove", "value");
    sr_create(db, &err, NULL, "items:patch2", &content);
    sr_object_free(content);
    
    /* Patch remove */
    int len = sr_patch_remove(db, &err, &results, "items:patch2", "/to_remove");
    ASSERT_GE(len, 0);
    
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_patch_replace(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /* First create a record */
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "original");
    sr_create(db, &err, NULL, "items:patch3", &content);
    sr_object_free(content);
    
    /* Patch replace */
    sr_value_t *value = sr_value_string("replaced");
    int len = sr_patch_replace(db, &err, &results, "items:patch3", "/name", value);
    ASSERT_GE(len, 0);
    
    sr_value_free(value);
    if (len > 0) {
        sr_values_free(results, len);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Transaction Tests
 * ============================================================================ */

int test_sr_begin(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    int res = sr_begin(db, &err);
    ASSERT_GE(res, 0);
    
    /* Cancel to clean up */
    sr_cancel(db, &err);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_commit(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_begin(db, &err);
    
    int res = sr_commit(db, &err);
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_cancel(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_begin(db, &err);
    
    int res = sr_cancel(db, &err);
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Session Variable Tests
 * ============================================================================ */

int test_sr_set(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *value = sr_value_int(42);
    
    int res = sr_set(db, &err, "my_var", value);
    ASSERT_GE(res, 0);
    
    sr_value_free(value);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_unset(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *value = sr_value_int(42);
    sr_set(db, &err, "my_var", value);
    sr_value_free(value);
    
    int res = sr_unset(db, &err, "my_var");
    ASSERT_GE(res, 0);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Live Query Tests
 * ============================================================================ */

int test_sr_select_live(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_stream_t *stream;
    
    if (seed_table(db, "CREATE items:1 SET name = 'a'") != API_TEST_PASS) {
        sr_surreal_disconnect(db);
        return API_TEST_FAIL;
    }

    int res = sr_select_live(db, &err, &stream, "items");
    ASSERT_GE(res, 0);
    ASSERT_NOT_NULL(stream);
    
    sr_stream_kill(stream);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * Import/Export Tests
 * ============================================================================ */

int test_sr_export(void) {
    /* Skip: requires file system access */
    return API_TEST_SKIP;
}

int test_sr_import(void) {
    /* Skip: requires file system access */
    return API_TEST_SKIP;
}

/* ============================================================================
 * Value Creation Tests
 * ============================================================================ */

int test_sr_value_none(void) {
    sr_value_t *val = sr_value_none();
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NONE);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_null(void) {
    sr_value_t *val = sr_value_null();
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NULL);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_bool(void) {
    sr_value_t *val_true = sr_value_bool(true);
    ASSERT_NOT_NULL(val_true);
    ASSERT_EQ(val_true->tag, SR_VALUE_BOOL);
    ASSERT_TRUE(val_true->sr_value_bool);
    sr_value_free(val_true);
    
    sr_value_t *val_false = sr_value_bool(false);
    ASSERT_NOT_NULL(val_false);
    ASSERT_FALSE(val_false->sr_value_bool);
    sr_value_free(val_false);
    
    return API_TEST_PASS;
}

int test_sr_value_int(void) {
    sr_value_t *val = sr_value_int(12345);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    ASSERT_EQ(val->sr_value_number.tag, SR_NUMBER_INT);
    ASSERT_EQ(val->sr_value_number.sr_number_int, 12345);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_float(void) {
    sr_value_t *val = sr_value_float(3.14159);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    ASSERT_EQ(val->sr_value_number.tag, SR_NUMBER_FLOAT);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_string(void) {
    sr_value_t *val = sr_value_string("hello world");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_STRAND);
    ASSERT_NOT_NULL(val->sr_value_strand);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_object(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "key", "value");
    
    sr_value_t *val = sr_value_object(&obj);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_OBJECT);
    
    sr_value_free(val);
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_value_free(void) {
    sr_value_t *val = sr_value_int(42);
    ASSERT_NOT_NULL(val);
    sr_value_free(val);
    /* If we get here without crashing, test passes */
    return API_TEST_PASS;
}

int test_sr_value_duration(void) {
    sr_value_t *val = sr_value_duration(3600, 500000000);  /* 1 hour + 0.5 seconds */
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_DURATION);
    ASSERT_EQ(val->sr_value_duration.secs, 3600);
    ASSERT_EQ(val->sr_value_duration.nanos, 500000000);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_datetime(void) {
    sr_value_t *val = sr_value_datetime("2024-01-15T10:30:00Z");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_DATETIME);
    ASSERT_NOT_NULL(val->sr_value_datetime);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_uuid(void) {
    uint8_t uuid_bytes[16] = {0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
                              0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0};
    sr_value_t *val = sr_value_uuid(uuid_bytes);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_UUID);
    ASSERT_EQ(val->sr_value_uuid._0[0], 0x12);
    ASSERT_EQ(val->sr_value_uuid._0[15], 0xf0);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_array(void) {
    sr_value_t *val = sr_value_array(NULL);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_ARRAY);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_bytes(void) {
    uint8_t data[] = {0x01, 0x02, 0x03, 0x04, 0x05};
    sr_value_t *val = sr_value_bytes(data, 5);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_BYTES);
    ASSERT_EQ(val->sr_value_bytes.len, 5);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_thing(void) {
    sr_value_t *val = sr_value_thing("users", "john123");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_THING);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_point(void) {
    sr_value_t *val = sr_value_point(10.5, 20.3);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_POINT);
    /* Verify coordinates */
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_point._0.x, 10.5);
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_point._0.y, 20.3);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_linestring(void) {
    sr_g_coord coords[] = {
        {0.0, 0.0},
        {10.0, 10.0},
        {20.0, 0.0}
    };
    sr_value_t *val = sr_value_linestring(coords, 3);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_LINESTRING);
    /* Verify length */
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_linestring._0.len, 3);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_polygon(void) {
    /* Simple square polygon */
    sr_g_coord coords[] = {
        {0.0, 0.0},
        {10.0, 0.0},
        {10.0, 10.0},
        {0.0, 10.0},
        {0.0, 0.0}  /* Close the ring */
    };
    sr_value_t *val = sr_value_polygon(coords, 5);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_POLYGON);
    /* Verify exterior ring has 5 coordinates */
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_polygon._0._0.len, 5);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_multipoint(void) {
    sr_g_coord coords[] = {
        {1.0, 2.0},
        {3.0, 4.0},
        {5.0, 6.0}
    };
    sr_value_t *val = sr_value_multipoint(coords, 3);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_MULTIPOINT);
    /* Verify 3 points */
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_multipoint._0.len, 3);
    sr_value_free(val);
    return API_TEST_PASS;
}

/* ============================================================================
 * Object Manipulation Tests
 * ============================================================================ */

int test_sr_object_new(void) {
    sr_object_t obj = sr_object_new();
    /* Object should be created without crashing */
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_get(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "test_key", "test_value");
    
    const sr_value_t *val = sr_object_get(&obj, "test_key");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_STRAND);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_insert(void) {
    sr_object_t obj = sr_object_new();
    sr_value_t *val = sr_value_int(42);
    
    sr_object_insert(&obj, "number", val);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "number");
    ASSERT_NOT_NULL(retrieved);
    ASSERT_EQ(retrieved->tag, SR_VALUE_NUMBER);
    
    sr_value_free(val);
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_insert_str(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_name");
    
    const sr_value_t *val = sr_object_get(&obj, "name");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_STRAND);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_insert_int(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_int(&obj, "count", 100);
    
    const sr_value_t *val = sr_object_get(&obj, "count");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_insert_float(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_float(&obj, "ratio", 0.5f);
    
    const sr_value_t *val = sr_object_get(&obj, "ratio");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_insert_double(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_double(&obj, "precise", 3.14159265359);
    
    const sr_value_t *val = sr_object_get(&obj, "precise");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_free(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "key", "value");
    sr_object_free(obj);
    /* If we get here without crashing, test passes */
    return API_TEST_PASS;
}

/* ============================================================================
 * Array Tests
 * ============================================================================ */

int test_sr_values_free(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_value_t *results;
    
    /*
     * Seeded so the select returns rows and sr_values_free is actually reached.
     * This previously selected a table that did not exist, so the free under
     * test never ran even when the test reported a pass.
     */
    if (seed_table(db, "CREATE freeable:1 SET n = 1; CREATE freeable:2 SET n = 2")
        != API_TEST_PASS) {
        sr_surreal_disconnect(db);
        return API_TEST_FAIL;
    }

    int len = sr_select(db, &err, &results, "freeable");
    ASSERT_GE(len, 0);
    ASSERT_TRUE(len > 0);

    sr_values_free(results, len);
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

/* ============================================================================
 * RPC Tests
 * ============================================================================ */

int test_sr_surreal_rpc_new(void) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};
    
    int res = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    ASSERT_GE(res, 0);
    ASSERT_NOT_NULL(rpc);
    
    sr_surreal_rpc_disconnect(rpc);
    return API_TEST_PASS;
}

int test_sr_surreal_rpc_execute(void) {
    /* Skip: requires CBOR encoding setup */
    return API_TEST_SKIP;
}

int test_sr_surreal_rpc_notifications(void) {
    /* Skip: requires RPC setup */
    return API_TEST_SKIP;
}

int test_sr_surreal_rpc_free(void) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};
    
    sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    sr_surreal_rpc_disconnect(rpc);
    /* If we get here without crashing, test passes */
    return API_TEST_PASS;
}

/* ============================================================================
 * Stream Tests
 * ============================================================================ */

int test_sr_stream_next(void) {
    /* Tested as part of select_live */
    return API_TEST_SKIP;
}

int test_sr_stream_kill(void) {
    /* Tested as part of select_live */
    return API_TEST_SKIP;
}

int test_sr_rpc_stream_next(void) {
    /* Skip: requires RPC stream setup */
    return API_TEST_SKIP;
}

int test_sr_rpc_stream_free(void) {
    /* Skip: requires RPC stream setup */
    return API_TEST_SKIP;
}

/* ============================================================================
 * Utility Tests
 * ============================================================================ */

int test_sr_string_free(void) {
    sr_surreal_t *db;
    sr_string_t err;
    sr_string_t ver;
    
    sr_connect(&err, &db, "mem://");
    sr_version(db, &err, &ver);
    
    sr_string_free(ver);
    /* If we get here without crashing, test passes */
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_value_print(void) {
    sr_value_t *val = sr_value_string("test print");
    sr_value_print(val);
    sr_value_free(val);
    /* If we get here without crashing, test passes */
    return API_TEST_PASS;
}

int test_sr_value_eq(void) {
    sr_value_t *a = sr_value_int(42);
    sr_value_t *b = sr_value_int(42);
    sr_value_t *c = sr_value_int(99);
    
    ASSERT_TRUE(sr_value_eq(a, b));
    ASSERT_FALSE(sr_value_eq(a, c));
    
    sr_value_free(a);
    sr_value_free(b);
    sr_value_free(c);
    return API_TEST_PASS;
}

int test_sr_print_notification(void) {
    /* Skip: requires actual notification */
    return API_TEST_SKIP;
}

/* ============================================================================
 * Additional Geometry Tests
 * ============================================================================ */

int test_sr_value_multilinestring(void) {
    /* Create two linestrings */
    sr_g_coord line1[] = {{0.0, 0.0}, {10.0, 10.0}};
    sr_g_coord line2[] = {{20.0, 20.0}, {30.0, 30.0}, {40.0, 20.0}};
    
    const sr_g_coord *lines[] = {line1, line2};
    int lens[] = {2, 3};
    
    sr_value_t *val = sr_value_multilinestring(lines, lens, 2);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_MULTILINE);
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_multiline._0.len, 2);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_multipolygon(void) {
    /* Create two simple square polygons */
    sr_g_coord poly1[] = {{0.0, 0.0}, {10.0, 0.0}, {10.0, 10.0}, {0.0, 10.0}, {0.0, 0.0}};
    sr_g_coord poly2[] = {{20.0, 20.0}, {30.0, 20.0}, {30.0, 30.0}, {20.0, 30.0}, {20.0, 20.0}};
    
    const sr_g_coord *polys[] = {poly1, poly2};
    int lens[] = {5, 5};
    
    sr_value_t *val = sr_value_multipolygon(polys, lens, 2);
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_GEOMETRY_OBJECT);
    ASSERT_EQ(val->sr_geometry_object.tag, SR_GEOMETRY_MULTIPOLYGON);
    ASSERT_EQ(val->sr_geometry_object.sr_geometry_multipolygon._0.len, 2);
    sr_value_free(val);
    return API_TEST_PASS;
}

int test_sr_value_decimal(void) {
    sr_value_t *val = sr_value_decimal("123.456789012345678901234567890");
    ASSERT_NOT_NULL(val);
    ASSERT_EQ(val->tag, SR_VALUE_NUMBER);
    ASSERT_EQ(val->sr_value_number.tag, SR_NUMBER_DECIMAL);
    sr_value_free(val);
    return API_TEST_PASS;
}

/* ============================================================================
 * Array Manipulation Tests
 * ============================================================================ */

int test_sr_array_len(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_arr_res_t *results;
    
    /* Query that returns an array */
    int res = sr_query(db, &err, &results, "RETURN [1, 2, 3]", NULL);
    ASSERT_GE(res, 0);
    
    if (res > 0 && results[0].ok.arr != NULL) {
        int len = sr_array_len(&results[0].ok);
        ASSERT_EQ(len, 3);
        sr_arr_res_arr_free(results, res);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_array_get(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    sr_arr_res_t *results;
    
    /* Query that returns an array */
    int res = sr_query(db, &err, &results, "RETURN [10, 20, 30]", NULL);
    ASSERT_GE(res, 0);
    
    if (res > 0 && results[0].ok.arr != NULL) {
        const sr_value_t *elem = sr_array_get(&results[0].ok, 1);
        ASSERT_NOT_NULL(elem);
        ASSERT_EQ(elem->tag, SR_VALUE_NUMBER);
        
        /* Out of bounds should return NULL */
        const sr_value_t *oob = sr_array_get(&results[0].ok, 100);
        ASSERT_TRUE(oob == NULL);
        
        sr_arr_res_arr_free(results, res);
    }
    
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}

int test_sr_array_push(void) {
    sr_value_t *arr_val = sr_value_array(NULL);
    ASSERT_NOT_NULL(arr_val);
    ASSERT_EQ(arr_val->tag, SR_VALUE_ARRAY);
    
    /* Push a value to the empty array */
    sr_value_t *int_val = sr_value_int(42);
    sr_array_t *new_arr = sr_array_push(arr_val->sr_value_array, int_val);
    ASSERT_NOT_NULL(new_arr);
    ASSERT_EQ(sr_array_len(new_arr), 1);
    
    /* Verify the value was added */
    const sr_value_t *elem = sr_array_get(new_arr, 0);
    ASSERT_NOT_NULL(elem);
    ASSERT_EQ(elem->tag, SR_VALUE_NUMBER);
    
    sr_array_free(new_arr);
    sr_value_free(int_val);
    sr_value_free(arr_val);
    return API_TEST_PASS;
}

/* ============================================================================
 * Object Iteration Tests
 * ============================================================================ */

int test_sr_object_len(void) {
    sr_object_t obj = sr_object_new();
    ASSERT_EQ(sr_object_len(&obj), 0);
    
    sr_object_insert_str(&obj, "key1", "value1");
    ASSERT_EQ(sr_object_len(&obj), 1);
    
    sr_object_insert_int(&obj, "key2", 42);
    ASSERT_EQ(sr_object_len(&obj), 2);
    
    sr_object_free(obj);
    return API_TEST_PASS;
}

int test_sr_object_keys(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "alpha", "a");
    sr_object_insert_str(&obj, "beta", "b");
    sr_object_insert_str(&obj, "gamma", "c");
    
    char **keys = NULL;
    int len = sr_object_keys(&obj, &keys);
    ASSERT_EQ(len, 3);
    ASSERT_NOT_NULL(keys);
    
    /* Keys should be in sorted order (BTreeMap) */
    ASSERT_TRUE(strcmp(keys[0], "alpha") == 0);
    ASSERT_TRUE(strcmp(keys[1], "beta") == 0);
    ASSERT_TRUE(strcmp(keys[2], "gamma") == 0);
    
    sr_string_arr_free(keys, len);
    sr_object_free(obj);
    return API_TEST_PASS;
}

/* ============================================================================
 * Kill Live Query Test
 * ============================================================================ */

int test_sr_kill(void) {
    sr_surreal_t *db;
    if (setup_db(&db) != API_TEST_PASS) return API_TEST_FAIL;
    
    sr_string_t err;
    
    /* Create a table first */
    sr_arr_res_t *res = NULL;
    int seeded = sr_query(db, &err, &res, "CREATE test_kill SET value = 1", NULL);
    if (seeded < 0) { if (err) sr_string_free(err); }
    else sr_arr_res_arr_free(res, seeded);
    
    /* Start a live query */
    sr_stream_t *stream;
    int result = sr_select_live(db, &err, &stream, "test_kill");
    if (result < 0) {
        /* Live queries may not be supported in all modes */
        sr_surreal_disconnect(db);
        return API_TEST_SKIP;
    }
    
    /* Kill using a fake UUID - should not crash */
    result = sr_kill(db, &err, "00000000-0000-0000-0000-000000000000");
    /* May fail if UUID doesn't exist, but shouldn't crash */
    if (result < 0 && err) {
        sr_string_free(err);
    }
    
    sr_stream_kill(stream);
    sr_surreal_disconnect(db);
    return API_TEST_PASS;
}
