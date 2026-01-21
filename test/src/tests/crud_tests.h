#ifndef CRUD_TESTS_H
#define CRUD_TESTS_H

#include <stdio.h>
#include <string.h>
#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_create(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_item");
    sr_object_insert_int(&obj, "value", 42);
    
    sr_object_t *result;
    int len = sr_create(db, &err, &result, "test_table:1", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "create should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "create should succeed");
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

static inline void test_sr_delete(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "to_delete");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:2", &obj);
    sr_free_object(obj);
    
    sr_value_t *delete_result;
    int len = sr_delete(db, &err, &delete_result, "test_table:2");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "delete should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "delete should succeed");
    
    if (len > 0) {
        sr_free_arr(delete_result, len);
    }
    
    test_helper_disconnect(db);
}

static inline void test_sr_insert(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "inserted_item");
    sr_object_insert_int(&obj, "count", 10);
    
    sr_value_t *result;
    int len = sr_insert(db, &err, &result, "test_table", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "insert should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "insert should succeed");
    
    if (len > 0) {
        sr_free_arr(result, len);
    }
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

static inline void test_sr_select(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "select_test");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:3", &obj);
    sr_free_object(obj);
    
    sr_value_t *select_result;
    int len = sr_select(db, &err, &select_result, "test_table:3");
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "select should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "select should succeed");
    
    if (len > 0) {
        sr_free_arr(select_result, len);
    }
    
    test_helper_disconnect(db);
}

static inline void test_sr_update(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:4", &obj);
    sr_free_object(obj);
    
    sr_object_t update_obj = sr_object_new();
    sr_object_insert_str(&update_obj, "name", "updated");
    sr_value_t *update_result;
    int len = sr_update(db, &err, &update_result, "test_table:4", &update_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "update should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(update_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "update should succeed");
    
    if (len > 0) {
        sr_free_arr(update_result, len);
    }
    
    sr_free_object(update_obj);
    test_helper_disconnect(db);
}

static inline void test_sr_upsert(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "upserted");
    
    sr_value_t *result;
    int len = sr_upsert(db, &err, &result, "test_table:5", &obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "upsert should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "upsert should succeed");
    
    if (len > 0) {
        sr_free_arr(result, len);
    }
    
    sr_free_object(obj);
    test_helper_disconnect(db);
}

static inline void test_sr_merge(void) {
    sr_surreal_t *db = test_helper_connect("memory");
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");
    test_helper_setup_ns_db(db, "test_ns", "test_db");
    
    sr_string_t err;
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "original");
    sr_object_insert_int(&obj, "value", 1);
    sr_object_t *create_result;
    sr_create(db, &err, &create_result, "test_table:6", &obj);
    sr_free_object(obj);
    
    sr_object_t merge_obj = sr_object_new();
    sr_object_insert_int(&merge_obj, "extra", 99);
    sr_value_t *merge_result;
    int len = sr_merge(db, &err, &merge_result, "test_table:6", &merge_obj);
    if (len < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "merge should succeed: %s", err);
        sr_free_string(err);
        sr_free_object(merge_obj);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "merge should succeed");
    
    if (len > 0) {
        sr_free_arr(merge_result, len);
    }
    
    sr_free_object(merge_obj);
    test_helper_disconnect(db);
}

#endif // CRUD_TESTS_H
