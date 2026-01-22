#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Utility);

TEST_SETUP(Utility) {
}

TEST_TEAR_DOWN(Utility) {
}

TEST(Utility, PrintNotification) {
    // Create a mock notification structure for testing
    sr_notification_t notification = {0};
    
    // Initialize with test data
    notification.action = SR_ACTION_CREATE;
    
    // Create a simple value for the notification data
    sr_value_t *val = sr_value_string("test notification data");
    notification.data = *val;
    
    // Test that print doesn't crash
    sr_print_notification(&notification);
    
    // Clean up
    sr_value_free(val);
    // Test passes if we get here without crashing
}

TEST(Utility, ValuePrint) {
    sr_value_t *val = sr_value_string("test print");
    sr_value_print(val);
    sr_value_free(val);
    // If we get here without crashing, test passes
}

TEST_GROUP_RUNNER(Utility) {
    RUN_TEST_CASE(Utility, PrintNotification);
    RUN_TEST_CASE(Utility, ValuePrint);
}
