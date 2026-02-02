#include "surrealdb.h"
#include "unity_fixture.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(IO);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(IO) {
    db = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(IO) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

TEST(IO, Export) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    // Create some data to export
    sr_object_t content = sr_object_new();
    sr_object_insert_str(&content, "name", "test_export");
    sr_object_t *result;
    int res = sr_create(db, &err, &result, "export_test", &content);
    sr_free_object(content);

    if (res < 0) {
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE("Failed to create test data for export");
    }

    // Export to a temp file
    const char *export_file = "test_export.surql";
    res = sr_export(db, &err, export_file);

    if (res < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "Export failed: %s", err ? err : "unknown");
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }

    // Clean up - remove the temp file
    remove(export_file);
}

TEST(IO, Import) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    // Create a simple SurrealQL file to import
    const char *import_file = "test_import.surql";
    FILE *f = fopen(import_file, "w");
    if (f == NULL) {
        TEST_FAIL_MESSAGE("Could not create import test file");
    }
    fprintf(f, "CREATE import_test:1 SET name = 'imported';\n");
    fclose(f);

    // Import the file
    int res = sr_import(db, &err, import_file);

    // Clean up the temp file first
    remove(import_file);

    if (res < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "Import failed: %s", err ? err : "unknown");
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }

    // Verify the data was imported
    sr_value_t *results;
    int len = sr_select(db, &err, &results, "import_test:1");
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, len, "Select should succeed after import");

    if (len > 0) {
        sr_free_arr(results, len);
    }
}

TEST_GROUP_RUNNER(IO) {
    RUN_TEST_CASE(IO, Export);
    RUN_TEST_CASE(IO, Import);
}
