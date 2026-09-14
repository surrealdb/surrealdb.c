#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(CRUD);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(CRUD) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(CRUD) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(CRUD, Create) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_item");
    sr_object_insert_int(&obj, "value", 42);
    
    sr_object_t result;
    int len = sr_create(db, &err, &result, "test_table:1", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "create should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(1, len, "create with a result slot reports one record");

    /* res_ptr was non-null, so the record is written by value and owned here.
       Freeing it must release everything: sr_create previously boxed the
       object and leaked the box, which no C function could reclaim. */
    sr_object_free(result);
    sr_object_free(obj);
}

TEST(CRUD, Delete) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "to_delete");
    /* The result is not inspected here; null discards it. */
    sr_create(db, &err, NULL, "test_table:2", &obj);
    sr_object_free(obj);
    
    sr_value_t *delete_result;
    int len = sr_delete(db, &err, &delete_result, "test_table:2");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "delete should succeed: %s", err);
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "delete should succeed");
    
    if (len > 0) {
        sr_values_free(delete_result, len);
    }
}

TEST(CRUD, Insert) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "inserted_item");
    sr_object_insert_int(&obj, "count", 10);
    
    sr_value_t *result;
    int len = sr_insert(db, &err, &result, "test_table", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "insert should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "insert should succeed");
    
    if (len > 0) {
        sr_values_free(result, len);
    }
    
    sr_object_free(obj);
}

TEST(CRUD, Select) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "select_test");
    /* The result is not inspected here; null discards it. */
    sr_create(db, &err, NULL, "test_table:3", &obj);
    sr_object_free(obj);
    
    sr_value_t *select_result;
    int len = sr_select(db, &err, &select_result, "test_table:3");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "select should succeed: %s", err);
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "select should succeed");
    
    if (len > 0) {
        sr_values_free(select_result, len);
    }
}

TEST(CRUD, Update) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    /* The result is not inspected here; null discards it. */
    sr_create(db, &err, NULL, "test_table:4", &obj);
    sr_object_free(obj);
    
    sr_object_t update_obj = sr_object_new();
    sr_object_insert_str(&update_obj, "name", "updated");
    
    sr_value_t *update_result;
    int len = sr_update(db, &err, &update_result, "test_table:4", &update_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "update should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(update_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "update should succeed");
    
    if (len > 0) {
        sr_values_free(update_result, len);
    }
    sr_object_free(update_obj);
}

TEST(CRUD, Upsert) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "upserted_item");
    
    sr_value_t *result;
    int len = sr_upsert(db, &err, &result, "test_table:5", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "upsert should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "upsert should succeed");
    
    if (len > 0) {
        sr_values_free(result, len);
    }
    sr_object_free(obj);
}

TEST(CRUD, Merge) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    sr_object_insert_int(&obj, "count", 1);
    /* The result is not inspected here; null discards it. */
    sr_create(db, &err, NULL, "test_table:6", &obj);
    sr_object_free(obj);
    
    sr_object_t merge_obj = sr_object_new();
    sr_object_insert_int(&merge_obj, "count", 2);
    
    sr_value_t *merge_result;
    int len = sr_merge(db, &err, &merge_result, "test_table:6", &merge_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "merge should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(merge_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "merge should succeed");
    
    if (len > 0) {
        sr_values_free(merge_result, len);
    }
    sr_object_free(merge_obj);
}

TEST(CRUD, InsertRelation) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    /* A relation needs both endpoints to exist first. */
    sr_object_t a = sr_object_new();
    sr_object_insert_str(&a, "name", "alice");
    sr_create(db, &err, NULL, "person:alice", &a);
    sr_object_free(a);

    sr_object_t b = sr_object_new();
    sr_object_insert_str(&b, "name", "bob");
    sr_create(db, &err, NULL, "person:bob", &b);
    sr_object_free(b);

    /* in and out must be record IDs, not strings. */
    sr_value_t *from = sr_value_thing("person", "alice");
    sr_value_t *to = sr_value_thing("person", "bob");

    sr_object_t edge = sr_object_new();
    sr_object_insert(&edge, "in", from);
    sr_object_insert(&edge, "out", to);
    sr_object_insert_str(&edge, "since", "2026");

    sr_value_t *result = NULL;
    int len = sr_insert_relation(db, &err, &result, "knows", &edge);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "insert_relation should succeed: %s", err);
        sr_string_free(err);
        sr_object_free(edge);
        sr_value_free(from);
        sr_value_free(to);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "insert_relation should succeed");

    sr_object_free(edge);
    sr_value_free(from);
    sr_value_free(to);
    if (result && len > 0) {
        sr_values_free(result, len);
    }
}

TEST_GROUP_RUNNER(CRUD) {
    RUN_TEST_CASE(CRUD, Create);
    RUN_TEST_CASE(CRUD, Delete);
    RUN_TEST_CASE(CRUD, Insert);
    RUN_TEST_CASE(CRUD, Select);
    RUN_TEST_CASE(CRUD, Update);
    RUN_TEST_CASE(CRUD, Upsert);
    RUN_TEST_CASE(CRUD, Merge);
    RUN_TEST_CASE(CRUD, InsertRelation);
}
