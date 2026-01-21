#ifndef OBJECT_TESTS_H
#define OBJECT_TESTS_H

#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_object_insert(void) {
    sr_object_t obj = sr_object_new();
    sr_value_t val = {
        .tag = SR_VALUE_NUMBER,
        .sr_value_number = {
            .tag = SR_NUMBER_INT,
            .sr_number_int = 42
        }
    };
    
    sr_object_insert(&obj, "test_key", &val);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "test_key");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    
    sr_free_object(obj);
}

static inline void test_sr_object_insert_str(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_str(&obj, "name", "test_string");
    
    const sr_value_t *retrieved = sr_object_get(&obj, "name");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_STRAND, retrieved->tag, "Value should be a string");
    
    sr_free_object(obj);
}

static inline void test_sr_object_insert_int(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_int(&obj, "count", 100);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "count");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_INT, retrieved->sr_value_number.tag, "Number should be an int");
    TEST_ASSERT_EQUAL_INT_MESSAGE(100, (int)retrieved->sr_value_number.sr_number_int, "Value should be 100");
    
    sr_free_object(obj);
}

static inline void test_sr_object_insert_float(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_float(&obj, "ratio", 3.14f);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "ratio");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_FLOAT, retrieved->sr_value_number.tag, "Number should be a float");
    
    sr_free_object(obj);
}

static inline void test_sr_object_insert_double(void) {
    sr_object_t obj = sr_object_new();
    sr_object_insert_double(&obj, "pi", 3.14159265359);
    
    const sr_value_t *retrieved = sr_object_get(&obj, "pi");
    TEST_ASSERT_NOT_NULL_MESSAGE(retrieved, "Retrieved value should not be null");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NUMBER, retrieved->tag, "Value should be a number");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_NUMBER_FLOAT, retrieved->sr_value_number.tag, "Number should be a float");
    
    sr_free_object(obj);
}

#endif // OBJECT_TESTS_H
