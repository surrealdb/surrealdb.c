/*
 * C driver for the test corpus.
 *
 * The group list lives in runner.c so that the Rust driver in c-tests/ runs the
 * same corpus without duplicating it. Accepts the usual Unity Fixture options,
 * so `test_runner -g CRUD` runs a single group — which is how CTest registers
 * one test per group.
 */

#include <stdio.h>
#include <stdlib.h>

#include "unity_fixture.h"

#include "test_support.h"

void sr_run_all_test_groups(void);

int main(int argc, const char *argv[])
{
    printf("=============================================================\n");
    printf("SurrealDB C API Test Suite (Unity Fixture Framework)\n");
    printf("=============================================================\n\n");

    /*
     * Per test, not for the suite: the countdown restarts whenever the running
     * test changes. Generous enough that a slow machine will not trip it, short
     * enough that CI fails rather than stalls.
     */
    unsigned timeout = 60;
    const char *configured = getenv("SURREALDB_TEST_TIMEOUT");
    if (configured != NULL) {
        long parsed = strtol(configured, NULL, 10);
        if (parsed > 0) {
            timeout = (unsigned)parsed;
        }
    }
    test_watchdog_start(timeout);
    int failures = UnityMain(argc, argv, sr_run_all_test_groups);
    test_watchdog_stop();

    return failures;
}
