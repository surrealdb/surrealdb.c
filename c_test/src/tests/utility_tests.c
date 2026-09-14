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
    
    /* Shallow-copied from the box on purpose: sr_value_string hands back a
       boxed Value and there is no way in C to move the value out of its box,
       so the notification here *aliases* data the box still owns. That makes
       sr_value_free(val) the single release point -- calling
       sr_notification_free as well would be a double free, and using
       notification.data after this point would be a use-after-free.
       A notification obtained from sr_stream_next owns its data outright and
       is released with sr_notification_free; see stream_tests.c. */
    sr_value_t *val = sr_value_string("test notification data");
    notification.data = *val;

    sr_print_notification(&notification);

    sr_value_free(val);
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
