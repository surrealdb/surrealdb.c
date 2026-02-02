#include "../../include/surrealdb.h"
#include <stdio.h>

void print_results(sr_arr_res_t *results, int num) {
    for (int i = 0; i < num; i++) {
        if (results[i].err.msg == NULL) {
            printf("  Statement %d: %d results\n", i + 1, results[i].ok.len);
            for (int j = 0; j < results[i].ok.len; j++) {
                printf("    ");
                sr_value_print(&results[i].ok.arr[j]);
            }
        } else {
            printf("  Statement %d ERROR: %s\n", i + 1, results[i].err.msg);
        }
    }
}

int main() {
    sr_string_t err = NULL;
    sr_surreal_t *db = NULL;
    sr_arr_res_t *r = NULL;
    int n;

    printf("=== Reproducing saas_example data creation ===\n\n");

    if (sr_connect(&err, &db, "mem://") < 0) {
        printf("Connect failed: %s\n", err);
        return 1;
    }
    sr_use_ns(db, &err, "saas_platform");
    sr_use_db(db, &err, "production");
    printf("[OK] Connected\n\n");

    // Create plan like saas_example does
    printf("1. Creating plan with sr_create:\n");
    sr_object_t plan = sr_object_new();
    sr_object_insert_str(&plan, "name", "Pro");
    sr_object_insert_int(&plan, "price_monthly", 99);
    sr_object_insert_int(&plan, "max_users", 50);

    sr_object_t *plan_result = NULL;
    int res = sr_create(db, &err, &plan_result, "plan:pro", &plan);
    if (res < 0) {
        printf("  ERROR: %s\n", err);
    } else {
        printf("  sr_create returned %d\n", res);
        if (plan_result) {
            printf("  Result: ");
            // Print the object somehow - need to convert to value
        }
    }
    if (plan_result)
        sr_free_object(*plan_result);
    sr_free_object(plan);

    // Verify plan exists
    printf("\n2. Verify plan exists:\n");
    n = sr_query(db, &err, &r, "SELECT * FROM plan;", NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    // Create organization like saas_example does
    printf("\n3. Creating organization with sr_create:\n");
    sr_object_t org = sr_object_new();
    sr_object_insert_str(&org, "name", "TechStartup Inc");
    sr_object_insert_str(&org, "industry", "Technology");
    sr_object_insert_str(&org, "status", "active");

    sr_object_t *org_result = NULL;
    res = sr_create(db, &err, &org_result, "organization:techstartup", &org);
    if (res < 0) {
        printf("  ERROR: %s\n", err);
    } else {
        printf("  sr_create returned %d\n", res);
    }
    if (org_result)
        sr_free_object(*org_result);
    sr_free_object(org);

    // Verify organization exists
    printf("\n4. Verify organization exists:\n");
    n = sr_query(db, &err, &r, "SELECT * FROM organization;", NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    // Create subscription relation
    printf("\n5. Creating subscription with sr_relate:\n");
    sr_object_t sub = sr_object_new();
    sr_object_insert_str(&sub, "started_at", "2024-01-15T10:00:00Z");
    sr_object_insert_str(&sub, "billing_cycle", "monthly");
    sr_object_insert_str(&sub, "status", "active");

    sr_value_t *rel_result = NULL;
    res = sr_relate(db, &err, &rel_result, "organization:techstartup", "subscribes_to", "plan:pro",
                    &sub);
    if (res < 0) {
        printf("  ERROR: %s\n", err);
    } else {
        printf("  sr_relate returned %d\n", res);
        if (rel_result && res > 0) {
            printf("  Relation: ");
            sr_value_print(&rel_result[0]);
        }
    }
    if (rel_result)
        sr_free_arr(rel_result, res);
    sr_free_object(sub);

    // Verify relation exists
    printf("\n6. Verify relation exists:\n");
    n = sr_query(db, &err, &r, "SELECT * FROM subscribes_to;", NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    // NOW test the exact saas_example step 9 query
    printf("\n7. Step 9 query - SELECT with graph traversal:\n");
    n = sr_query(db, &err, &r,
                 "SELECT name, industry, status, "
                 "->subscribes_to->plan.name AS plan_name, "
                 "->subscribes_to->plan.price_monthly AS monthly_cost "
                 "FROM organization;",
                 NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    // Simpler graph query
    printf("\n8. Simpler: SELECT name, ->subscribes_to->plan AS plans FROM organization:\n");
    n = sr_query(db, &err, &r, "SELECT name, ->subscribes_to->plan AS plans FROM organization;",
                 NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    // Even simpler
    printf("\n9. Just traversal: SELECT ->subscribes_to FROM organization:\n");
    n = sr_query(db, &err, &r, "SELECT ->subscribes_to FROM organization;", NULL);
    print_results(r, n);
    sr_free_arr_res_arr(r, n);

    sr_surreal_disconnect(db);
    printf("\n=== Test Complete ===\n");
    return 0;
}
