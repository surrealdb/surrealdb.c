#ifndef TEST_HELPERS_H
#define TEST_HELPERS_H

#include "../../../include/surrealdb.h"
#include <stdio.h>
#include <stdlib.h>

// Helper to create a test database connection
static inline sr_surreal_t *test_helper_connect(const char *endpoint) {
    sr_surreal_t *db;
    sr_string_t err;

    if (sr_connect(&err, &db, endpoint) < 0) {
        printf("ERROR: Failed to connect to %s: %s\n", endpoint, err);
        sr_free_string(err);
        return NULL;
    }

    return db;
}

// Helper to setup namespace and database
static inline int test_helper_setup_ns_db(sr_surreal_t *db, const char *ns, const char *db_name) {
    sr_string_t err;

    if (sr_use_ns(db, &err, ns) < 0) {
        printf("ERROR: Failed to use namespace %s: %s\n", ns, err);
        sr_free_string(err);
        return -1;
    }

    if (sr_use_db(db, &err, db_name) < 0) {
        printf("ERROR: Failed to use database %s: %s\n", db_name, err);
        sr_free_string(err);
        return -1;
    }

    return 0;
}

// Helper to cleanup connection
static inline void test_helper_disconnect(sr_surreal_t *db) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
    }
}

#endif // TEST_HELPERS_H
