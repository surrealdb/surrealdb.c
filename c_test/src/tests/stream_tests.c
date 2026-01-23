#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Stream);

TEST_SETUP(Stream) {
}

TEST_TEAR_DOWN(Stream) {
}

TEST(Stream, Next) {
    TEST_IGNORE_MESSAGE("Stream next test is covered by select_live test");
}

TEST(Stream, Kill) {
    TEST_IGNORE_MESSAGE("Stream kill test is covered by select_live test");
}

TEST_GROUP_RUNNER(Stream) {
    RUN_TEST_CASE(Stream, Next);
    RUN_TEST_CASE(Stream, Kill);
}
