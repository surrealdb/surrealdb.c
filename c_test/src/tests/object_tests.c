#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Object);

TEST_SETUP(Object) {
}

TEST_TEAR_DOWN(Object) {
}

TEST(Object, Insert) {
    sr_object_t obj = sr_object_new();
    sr_value_t *val = sr_value_int(42);
    
    sr_object_insert(&obj, "number", val);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "number");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    
    sr_value_free(val);
    sr_free_object(obj);
}

TEST(Object, InsertStr) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_name");
    
    const sr_value_t *val = sr_object_get(&obj, "name");
    TEST_ASSERT_NOT_NULL_MESSAGE(val, "Retrieved value should not be NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_STRAND, val->tag, "Value should be a string");
    
    sr_free_object(obj);
}

TEST(Object, InsertInt) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_int(&obj, "count", 100);
    
    const sr_value_t *val = sr_object_get(&obj, "count");
    TEST_ASSERT_NOT_NULL_MESSAGE(val, "Retrieved value should not be NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, val->tag, "Value should be a number");
    
    sr_free_object(obj);
}

TEST(Object, InsertFloat) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_float(&obj, "ratio", 0.5f);
    
    const sr_value_t *val = sr_object_get(&obj, "ratio");
    TEST_ASSERT_NOT_NULL_MESSAGE(val, "Retrieved value should not be NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, val->tag, "Value should be a number");
    
    sr_free_object(obj);
}

TEST(Object, InsertDouble) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_double(&obj, "precise", 3.14159265359);
    
    const sr_value_t *val = sr_object_get(&obj, "precise");
    TEST_ASSERT_NOT_NULL_MESSAGE(val, "Retrieved value should not be NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, val->tag, "Value should be a number");
    
    sr_free_object(obj);
}

TEST_GROUP_RUNNER(Object) {
    RUN_TEST_CASE(Object, Insert);
    RUN_TEST_CASE(Object, InsertStr);
    RUN_TEST_CASE(Object, InsertInt);
    RUN_TEST_CASE(Object, InsertFloat);
    RUN_TEST_CASE(Object, InsertDouble);
}
