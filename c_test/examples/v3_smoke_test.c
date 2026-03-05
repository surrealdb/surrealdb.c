#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "surrealdb.h"

int main(void) {
    sr_string_t err = NULL;
    sr_surreal_t *db = NULL;
    int failures = 0;

    printf("=== SurrealDB v3 Smoke Test ===\n\n");

    /* 1. Connect to in-memory database */
    if (sr_connect(&err, &db, "mem://") < 0) {
        printf("FAIL [connect]: %s\n", err);
        sr_free_string(err);
        return 1;
    }
    printf("OK   [connect mem://]\n");

    /* 2. Select namespace and database */
    if (sr_use_ns(db, &err, "test") < 0) {
        printf("FAIL [use_ns]: %s\n", err);
        sr_free_string(err);
        failures++;
    } else {
        printf("OK   [use_ns test]\n");
    }

    if (sr_use_db(db, &err, "test") < 0) {
        printf("FAIL [use_db]: %s\n", err);
        sr_free_string(err);
        failures++;
    } else {
        printf("OK   [use_db test]\n");
    }

    /* 3. Get version */
    {
        sr_string_t ver = NULL;
        int rc = sr_version(db, &err, &ver);
        if (rc < 0) {
            printf("FAIL [version]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [version]: %s\n", ver);
            sr_free_string(ver);
        }
    }

    /* 4. Health check */
    if (sr_health(db, &err) < 0) {
        printf("FAIL [health]: %s\n", err);
        sr_free_string(err);
        err = NULL;
        failures++;
    } else {
        printf("OK   [health]\n");
    }

    /* 5. Create a record */
    {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "name", "Alice");
        sr_object_insert_int(&content, "age", 30);
        sr_object_insert_str(&content, "email", "alice@example.com");

        sr_object_t *result = NULL;
        int rc = sr_create(db, &err, &result, "person:alice", &content);
        if (rc < 0) {
            printf("FAIL [create person:alice]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [create person:alice]\n");
            if (result) sr_free_object(*result);
        }
        sr_free_object(content);
    }

    /* 6. Create another record */
    {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "name", "Bob");
        sr_object_insert_int(&content, "age", 25);
        sr_object_insert_str(&content, "email", "bob@example.com");

        sr_object_t *result = NULL;
        int rc = sr_create(db, &err, &result, "person:bob", &content);
        if (rc < 0) {
            printf("FAIL [create person:bob]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [create person:bob]\n");
            if (result) sr_free_object(*result);
        }
        sr_free_object(content);
    }

    /* 7. Select all persons */
    {
        sr_value_t *persons = NULL;
        int len = sr_select(db, &err, &persons, "person");
        if (len < 0) {
            printf("FAIL [select person]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [select person]: got %d records\n", len);
            for (int i = 0; i < len; i++) {
                sr_value_print(&persons[i]);
            }
            sr_free_arr(persons, len);
        }
    }

    /* 8. Select a specific record */
    {
        sr_value_t *persons = NULL;
        int len = sr_select(db, &err, &persons, "person:alice");
        if (len < 0) {
            printf("FAIL [select person:alice]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [select person:alice]: got %d records\n", len);
            sr_free_arr(persons, len);
        }
    }

    /* 9. Run a query */
    {
        sr_arr_res_t *results = NULL;
        int num_stmts = sr_query(db, &err, &results, "SELECT * FROM person WHERE age > 20", NULL);
        if (num_stmts < 0) {
            printf("FAIL [query]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [query]: %d statement(s) returned\n", num_stmts);
            for (int i = 0; i < num_stmts; i++) {
                if (results[i].err.code == 0) {
                    printf("  stmt %d: %d results\n", i, results[i].ok.len);
                    for (int j = 0; j < results[i].ok.len; j++) {
                        sr_value_print(&results[i].ok.arr[j]);
                    }
                } else {
                    printf("  stmt %d: ERROR: %s\n", i, results[i].err.msg);
                }
            }
            sr_free_arr_res_arr(results, num_stmts);
        }
    }

    /* 10. Update a record with merge */
    {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "city", "New York");

        sr_value_t *merged = NULL;
        int len = sr_merge(db, &err, &merged, "person:alice", &content);
        if (len < 0) {
            printf("FAIL [merge person:alice]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [merge person:alice]: updated %d records\n", len);
            sr_free_arr(merged, len);
        }
        sr_free_object(content);
    }

    /* 11. Delete a record */
    {
        sr_value_t *deleted = NULL;
        int len = sr_delete(db, &err, &deleted, "person:bob");
        if (len < 0) {
            printf("FAIL [delete person:bob]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [delete person:bob]: deleted %d records\n", len);
            sr_free_arr(deleted, len);
        }
    }

    /* 12. Set and use a session variable */
    {
        sr_value_t *val = sr_value_string("test_value");
        int rc = sr_set(db, &err, "my_var", val);
        sr_value_free(val);
        if (rc < 0) {
            printf("FAIL [set variable]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [set variable]\n");
        }
    }

    /* 13. Unset the variable */
    {
        int rc = sr_unset(db, &err, "my_var");
        if (rc < 0) {
            printf("FAIL [unset variable]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [unset variable]\n");
        }
    }

    /* 14. Upsert a record */
    {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "name", "Charlie");
        sr_object_insert_int(&content, "age", 35);

        sr_value_t *upserted = NULL;
        int len = sr_upsert(db, &err, &upserted, "person:charlie", &content);
        if (len < 0) {
            printf("FAIL [upsert person:charlie]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [upsert person:charlie]: %d records\n", len);
            sr_free_arr(upserted, len);
        }
        sr_free_object(content);
    }

    /* 15. Create a relation */
    {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "since", "2024-01-01");

        sr_value_t *result = NULL;
        int len = sr_relate(db, &err, &result, "person:alice", "knows", "person:charlie", &content);
        if (len < 0) {
            printf("FAIL [relate]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [relate alice->knows->charlie]: %d records\n", len);
            sr_free_arr(result, len);
        }
        sr_free_object(content);
    }

    /* 16. Final verification query with graph traversal */
    {
        sr_arr_res_t *results = NULL;
        int num_stmts = sr_query(db, &err, &results,
            "SELECT *, ->knows->person AS friends FROM person:alice", NULL);
        if (num_stmts < 0) {
            printf("FAIL [graph query]: %s\n", err);
            sr_free_string(err);
            err = NULL;
            failures++;
        } else {
            printf("OK   [graph query]: %d statement(s)\n", num_stmts);
            for (int i = 0; i < num_stmts; i++) {
                if (results[i].err.code == 0) {
                    for (int j = 0; j < results[i].ok.len; j++) {
                        sr_value_print(&results[i].ok.arr[j]);
                    }
                }
            }
            sr_free_arr_res_arr(results, num_stmts);
        }
    }

    sr_surreal_disconnect(db);

    printf("\n=== Results: %d failure(s) ===\n", failures);
    return failures > 0 ? 1 : 0;
}
