#ifndef UTILITY_TESTS_H
#define UTILITY_TESTS_H

#include <unity.h>
#include "test_helpers.h"

static inline void test_sr_print_notification(void) {
    TEST_IGNORE_MESSAGE("Print notification test not yet implemented");
}

static inline void test_sr_value_print(void) {
    sr_value_t val = {
        .tag = SR_VALUE_NUMBER,
        .sr_value_number = {
            .tag = SR_NUMBER_INT,
            .sr_number_int = 42
        }
    };
    
    sr_value_print(&val);
    TEST_PASS_MESSAGE("sr_value_print should not crash");
}

#endif // UTILITY_TESTS_H
