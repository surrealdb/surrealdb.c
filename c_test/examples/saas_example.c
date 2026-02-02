/**
 * SaaS Platform Example - SurrealDB C FFI
 *
 * This example demonstrates a real-world SaaS (Software as a Service) platform
 * data model using the SurrealDB C FFI. It showcases:
 *
 * - Multi-tenant architecture with organizations
 * - User management with roles and permissions
 * - Subscription plans and billing
 * - Feature flags and entitlements
 * - Usage tracking and analytics
 * - Graph relations between entities
 * - Complex queries with aggregations
 *
 * Tables:
 *   - organization: Tenant/company accounts
 *   - user: Users belonging to organizations
 *   - plan: Subscription plans (free, starter, pro, enterprise)
 *   - feature: Available platform features
 *   - usage: Usage tracking records
 *   - member_of: Relation linking users to organizations with roles
 *   - subscribes_to: Relation linking organizations to plans
 *   - has_access: Relation linking plans to features
 */

#include "surrealdb.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Helper macro to check for errors and exit if operation failed
#define CHECK_ERROR(result, err, msg)                                     \
    do {                                                                  \
        if (result < 0) {                                                 \
            printf("ERROR: %s - %s\n", msg, err ? err : "Unknown error"); \
            if (err)                                                      \
                sr_free_string(err);                                      \
            if (db)                                                       \
                sr_surreal_disconnect(db);                                \
            return 1;                                                     \
        }                                                                 \
    } while (0)

// Helper function to print a separator line
void print_separator(const char *title) {
    printf("\n");
    printf("========================================================================\n");
    printf("  %s\n", title);
    printf("========================================================================\n");
}

// Helper function to print query results
void print_query_results(sr_arr_res_t *results, int num_statements) {
    for (int stmt = 0; stmt < num_statements; stmt++) {
        if (results[stmt].err.msg == NULL) {
            printf("  Statement %d: %d result(s)\n", stmt + 1, results[stmt].ok.len);
            for (int i = 0; i < results[stmt].ok.len; i++) {
                printf("    ");
                sr_value_print(&results[stmt].ok.arr[i]);
            }
        } else {
            printf("  Statement %d ERROR: %s\n", stmt + 1, results[stmt].err.msg);
        }
    }
}

int main(void) {
    printf("\n");
    printf("+========================================================================+\n");
    printf("|          SaaS Platform Example - SurrealDB C FFI                       |\n");
    printf("|                                                                        |\n");
    printf("|  Demonstrating multi-tenant SaaS data modeling with:                   |\n");
    printf("|  * Organizations & Users       * Subscription Plans                    |\n");
    printf("|  * Feature Entitlements        * Usage Tracking                        |\n");
    printf("|  * Graph Relations             * Complex Queries                       |\n");
    printf("+========================================================================+\n\n");

    sr_string_t err = NULL;
    sr_surreal_t *db = NULL;
    int result;

    // ========================================================================
    // Step 1: Connect to in-memory database
    // ========================================================================
    print_separator("Step 1: Connecting to Database");

    result = sr_connect(&err, &db, "mem://");
    CHECK_ERROR(result, err, "Failed to connect");
    printf("  [OK] Connected to in-memory database\n");

    result = sr_use_ns(db, &err, "saas_platform");
    CHECK_ERROR(result, err, "Failed to set namespace");
    printf("  [OK] Namespace: saas_platform\n");

    result = sr_use_db(db, &err, "production");
    CHECK_ERROR(result, err, "Failed to set database");
    printf("  [OK] Database: production\n");

    // ========================================================================
    // Step 2: Create Subscription Plans
    // ========================================================================
    print_separator("Step 2: Creating Subscription Plans");

    // Free Plan
    sr_object_t plan_free = sr_object_new();
    sr_object_insert_str(&plan_free, "name", "Free");
    sr_object_insert_int(&plan_free, "price_monthly", 0);
    sr_object_insert_int(&plan_free, "max_users", 3);
    sr_object_insert_int(&plan_free, "max_projects", 2);
    sr_object_insert_int(&plan_free, "storage_gb", 1);
    sr_object_insert_str(&plan_free, "support_level", "community");

    sr_object_t *plan_free_result = NULL;
    result = sr_create(db, &err, &plan_free_result, "plan:free", &plan_free);
    CHECK_ERROR(result, err, "Failed to create free plan");
    printf("  [OK] Created plan: Free ($0/mo, 3 users, 1GB storage)\n");
    if (plan_free_result)
        sr_free_object(*plan_free_result);
    sr_free_object(plan_free);

    // Starter Plan
    sr_object_t plan_starter = sr_object_new();
    sr_object_insert_str(&plan_starter, "name", "Starter");
    sr_object_insert_int(&plan_starter, "price_monthly", 29);
    sr_object_insert_int(&plan_starter, "max_users", 10);
    sr_object_insert_int(&plan_starter, "max_projects", 10);
    sr_object_insert_int(&plan_starter, "storage_gb", 25);
    sr_object_insert_str(&plan_starter, "support_level", "email");

    sr_object_t *plan_starter_result = NULL;
    result = sr_create(db, &err, &plan_starter_result, "plan:starter", &plan_starter);
    CHECK_ERROR(result, err, "Failed to create starter plan");
    printf("  [OK] Created plan: Starter ($29/mo, 10 users, 25GB storage)\n");
    if (plan_starter_result)
        sr_free_object(*plan_starter_result);
    sr_free_object(plan_starter);

    // Pro Plan
    sr_object_t plan_pro = sr_object_new();
    sr_object_insert_str(&plan_pro, "name", "Pro");
    sr_object_insert_int(&plan_pro, "price_monthly", 99);
    sr_object_insert_int(&plan_pro, "max_users", 50);
    sr_object_insert_int(&plan_pro, "max_projects", 100);
    sr_object_insert_int(&plan_pro, "storage_gb", 100);
    sr_object_insert_str(&plan_pro, "support_level", "priority");

    sr_object_t *plan_pro_result = NULL;
    result = sr_create(db, &err, &plan_pro_result, "plan:pro", &plan_pro);
    CHECK_ERROR(result, err, "Failed to create pro plan");
    printf("  [OK] Created plan: Pro ($99/mo, 50 users, 100GB storage)\n");
    if (plan_pro_result)
        sr_free_object(*plan_pro_result);
    sr_free_object(plan_pro);

    // Enterprise Plan
    sr_object_t plan_enterprise = sr_object_new();
    sr_object_insert_str(&plan_enterprise, "name", "Enterprise");
    sr_object_insert_int(&plan_enterprise, "price_monthly", 499);
    sr_object_insert_int(&plan_enterprise, "max_users", -1);    // unlimited
    sr_object_insert_int(&plan_enterprise, "max_projects", -1); // unlimited
    sr_object_insert_int(&plan_enterprise, "storage_gb", 1000);
    sr_object_insert_str(&plan_enterprise, "support_level", "dedicated");

    sr_object_t *plan_enterprise_result = NULL;
    result = sr_create(db, &err, &plan_enterprise_result, "plan:enterprise", &plan_enterprise);
    CHECK_ERROR(result, err, "Failed to create enterprise plan");
    printf("  [OK] Created plan: Enterprise ($499/mo, unlimited users, 1TB storage)\n");
    if (plan_enterprise_result)
        sr_free_object(*plan_enterprise_result);
    sr_free_object(plan_enterprise);

    // ========================================================================
    // Step 3: Create Platform Features
    // ========================================================================
    print_separator("Step 3: Creating Platform Features");

    const char *features[][4] = {
        {"feature:dashboard", "Dashboard", "Basic analytics dashboard", "free"},
        {"feature:api_access", "API Access", "REST and GraphQL API access", "starter"},
        {"feature:webhooks", "Webhooks", "Real-time webhook notifications", "starter"},
        {"feature:sso", "Single Sign-On", "SAML and OAuth SSO integration", "pro"},
        {"feature:audit_log", "Audit Logging", "Complete audit trail of all actions", "pro"},
        {"feature:custom_domain", "Custom Domain", "Use your own domain", "pro"},
        {"feature:sla", "SLA Guarantee", "99.99% uptime SLA", "enterprise"},
        {"feature:dedicated_support", "Dedicated Support", "24/7 dedicated support team",
         "enterprise"},
    };

    for (int i = 0; i < 8; i++) {
        sr_object_t feature = sr_object_new();
        sr_object_insert_str(&feature, "name", features[i][1]);
        sr_object_insert_str(&feature, "description", features[i][2]);
        sr_object_insert_str(&feature, "min_plan", features[i][3]);

        sr_object_t *feature_result = NULL;
        result = sr_create(db, &err, &feature_result, features[i][0], &feature);
        CHECK_ERROR(result, err, "Failed to create feature");
        printf("  [OK] Created feature: %s (requires %s plan)\n", features[i][1], features[i][3]);
        if (feature_result)
            sr_free_object(*feature_result);
        sr_free_object(feature);
    }

    // ========================================================================
    // Step 4: Create Organizations (Tenants)
    // ========================================================================
    print_separator("Step 4: Creating Organizations");

    // Organization 1: TechStartup Inc
    sr_object_t org1 = sr_object_new();
    sr_object_insert_str(&org1, "name", "TechStartup Inc");
    sr_object_insert_str(&org1, "slug", "techstartup");
    sr_object_insert_str(&org1, "industry", "Technology");
    sr_object_insert_str(&org1, "country", "USA");
    sr_object_insert_str(&org1, "created_at", "2024-01-15T10:00:00Z");
    sr_object_insert_str(&org1, "status", "active");

    sr_object_t *org1_result = NULL;
    result = sr_create(db, &err, &org1_result, "organization:techstartup", &org1);
    CHECK_ERROR(result, err, "Failed to create organization");
    printf("  [OK] Created organization: TechStartup Inc (Technology, USA)\n");
    if (org1_result)
        sr_free_object(*org1_result);
    sr_free_object(org1);

    // Organization 2: GlobalCorp
    sr_object_t org2 = sr_object_new();
    sr_object_insert_str(&org2, "name", "GlobalCorp");
    sr_object_insert_str(&org2, "slug", "globalcorp");
    sr_object_insert_str(&org2, "industry", "Finance");
    sr_object_insert_str(&org2, "country", "UK");
    sr_object_insert_str(&org2, "created_at", "2023-06-20T14:30:00Z");
    sr_object_insert_str(&org2, "status", "active");

    sr_object_t *org2_result = NULL;
    result = sr_create(db, &err, &org2_result, "organization:globalcorp", &org2);
    CHECK_ERROR(result, err, "Failed to create organization");
    printf("  [OK] Created organization: GlobalCorp (Finance, UK)\n");
    if (org2_result)
        sr_free_object(*org2_result);
    sr_free_object(org2);

    // Organization 3: LocalBiz
    sr_object_t org3 = sr_object_new();
    sr_object_insert_str(&org3, "name", "LocalBiz");
    sr_object_insert_str(&org3, "slug", "localbiz");
    sr_object_insert_str(&org3, "industry", "Retail");
    sr_object_insert_str(&org3, "country", "Canada");
    sr_object_insert_str(&org3, "created_at", "2024-03-01T09:15:00Z");
    sr_object_insert_str(&org3, "status", "trial");

    sr_object_t *org3_result = NULL;
    result = sr_create(db, &err, &org3_result, "organization:localbiz", &org3);
    CHECK_ERROR(result, err, "Failed to create organization");
    printf("  [OK] Created organization: LocalBiz (Retail, Canada) [TRIAL]\n");
    if (org3_result)
        sr_free_object(*org3_result);
    sr_free_object(org3);

    // ========================================================================
    // Step 5: Create Users
    // ========================================================================
    print_separator("Step 5: Creating Users");

    typedef struct {
        const char *id;
        const char *name;
        const char *email;
        const char *title;
        const char *created;
    } UserData;

    UserData users[] = {
        {"user:alice", "Alice Chen", "alice@techstartup.io", "CEO", "2024-01-15T10:05:00Z"},
        {"user:bob", "Bob Williams", "bob@techstartup.io", "CTO", "2024-01-16T11:20:00Z"},
        {"user:carol", "Carol Davis", "carol@techstartup.io", "Lead Developer",
         "2024-01-20T09:00:00Z"},
        {"user:david", "David Smith", "david@globalcorp.com", "VP Engineering",
         "2023-06-20T15:00:00Z"},
        {"user:eve", "Eve Johnson", "eve@globalcorp.com", "Security Lead", "2023-07-01T08:30:00Z"},
        {"user:frank", "Frank Miller", "frank@globalcorp.com", "DevOps Engineer",
         "2023-08-15T10:45:00Z"},
        {"user:grace", "Grace Lee", "grace@globalcorp.com", "Data Analyst", "2023-09-01T14:00:00Z"},
        {"user:henry", "Henry Brown", "henry@localbiz.ca", "Owner", "2024-03-01T09:20:00Z"},
    };

    for (int i = 0; i < 8; i++) {
        sr_object_t user = sr_object_new();
        sr_object_insert_str(&user, "name", users[i].name);
        sr_object_insert_str(&user, "email", users[i].email);
        sr_object_insert_str(&user, "title", users[i].title);
        sr_object_insert_str(&user, "created_at", users[i].created);
        sr_object_insert_str(&user, "status", "active");

        sr_object_t *user_result = NULL;
        result = sr_create(db, &err, &user_result, users[i].id, &user);
        CHECK_ERROR(result, err, "Failed to create user");
        printf("  [OK] Created user: %s (%s)\n", users[i].name, users[i].email);
        if (user_result)
            sr_free_object(*user_result);
        sr_free_object(user);
    }

    // ========================================================================
    // Step 6: Create Relations - Organization Memberships
    // ========================================================================
    print_separator("Step 6: Creating Organization Memberships (Graph Relations)");

    typedef struct {
        const char *user;
        const char *org;
        const char *role;
    } MembershipData;

    MembershipData memberships[] = {
        {"user:alice", "organization:techstartup", "owner"},
        {"user:bob", "organization:techstartup", "admin"},
        {"user:carol", "organization:techstartup", "member"},
        {"user:david", "organization:globalcorp", "owner"},
        {"user:eve", "organization:globalcorp", "admin"},
        {"user:frank", "organization:globalcorp", "member"},
        {"user:grace", "organization:globalcorp", "member"},
        {"user:henry", "organization:localbiz", "owner"},
    };

    for (int i = 0; i < 8; i++) {
        sr_object_t content = sr_object_new();
        sr_object_insert_str(&content, "role", memberships[i].role);
        sr_object_insert_str(&content, "joined_at", "2024-01-01T00:00:00Z");

        sr_value_t *rel_result = NULL;
        result = sr_relate(db, &err, &rel_result, memberships[i].user, "member_of",
                           memberships[i].org, &content);
        CHECK_ERROR(result, err, "Failed to create membership relation");
        printf("  [OK] %s -[member_of {role: %s}]-> %s\n", memberships[i].user, memberships[i].role,
               memberships[i].org);
        if (rel_result)
            sr_free_arr(rel_result, result);
        sr_free_object(content);
    }

    // ========================================================================
    // Step 7: Create Relations - Organization Subscriptions
    // ========================================================================
    print_separator("Step 7: Creating Subscription Relations");

    // TechStartup subscribes to Pro plan
    sr_object_t sub1 = sr_object_new();
    sr_object_insert_str(&sub1, "started_at", "2024-01-15T10:00:00Z");
    sr_object_insert_str(&sub1, "billing_cycle", "monthly");
    sr_object_insert_str(&sub1, "status", "active");

    sr_value_t *sub1_result = NULL;
    result = sr_relate(db, &err, &sub1_result, "organization:techstartup", "subscribes_to",
                       "plan:pro", &sub1);
    CHECK_ERROR(result, err, "Failed to create subscription");
    printf("  [OK] TechStartup Inc -[subscribes_to]-> Pro Plan ($99/mo)\n");
    if (sub1_result)
        sr_free_arr(sub1_result, result);
    sr_free_object(sub1);

    // GlobalCorp subscribes to Enterprise plan
    sr_object_t sub2 = sr_object_new();
    sr_object_insert_str(&sub2, "started_at", "2023-06-20T14:30:00Z");
    sr_object_insert_str(&sub2, "billing_cycle", "annual");
    sr_object_insert_str(&sub2, "status", "active");

    sr_value_t *sub2_result = NULL;
    result = sr_relate(db, &err, &sub2_result, "organization:globalcorp", "subscribes_to",
                       "plan:enterprise", &sub2);
    CHECK_ERROR(result, err, "Failed to create subscription");
    printf("  [OK] GlobalCorp -[subscribes_to]-> Enterprise Plan ($499/mo, annual)\n");
    if (sub2_result)
        sr_free_arr(sub2_result, result);
    sr_free_object(sub2);

    // LocalBiz subscribes to Free plan (trial)
    sr_object_t sub3 = sr_object_new();
    sr_object_insert_str(&sub3, "started_at", "2024-03-01T09:15:00Z");
    sr_object_insert_str(&sub3, "billing_cycle", "none");
    sr_object_insert_str(&sub3, "status", "trial");

    sr_value_t *sub3_result = NULL;
    result = sr_relate(db, &err, &sub3_result, "organization:localbiz", "subscribes_to",
                       "plan:free", &sub3);
    CHECK_ERROR(result, err, "Failed to create subscription");
    printf("  [OK] LocalBiz -[subscribes_to]-> Free Plan (trial)\n");
    if (sub3_result)
        sr_free_arr(sub3_result, result);
    sr_free_object(sub3);

    // ========================================================================
    // Step 8: Create Usage Records
    // ========================================================================
    print_separator("Step 8: Creating Usage Records");

    typedef struct {
        const char *org;
        const char *metric;
        int value;
        const char *period;
    } UsageData;

    UsageData usage_records[] = {
        {"organization:techstartup", "api_calls", 45230, "2024-01"},
        {"organization:techstartup", "api_calls", 52100, "2024-02"},
        {"organization:techstartup", "api_calls", 61500, "2024-03"},
        {"organization:techstartup", "storage_mb", 12500, "2024-03"},
        {"organization:techstartup", "active_users", 3, "2024-03"},
        {"organization:globalcorp", "api_calls", 1250000, "2024-01"},
        {"organization:globalcorp", "api_calls", 1380000, "2024-02"},
        {"organization:globalcorp", "api_calls", 1520000, "2024-03"},
        {"organization:globalcorp", "storage_mb", 450000, "2024-03"},
        {"organization:globalcorp", "active_users", 4, "2024-03"},
        {"organization:localbiz", "api_calls", 150, "2024-03"},
        {"organization:localbiz", "storage_mb", 50, "2024-03"},
        {"organization:localbiz", "active_users", 1, "2024-03"},
    };

    for (int i = 0; i < 13; i++) {
        sr_object_t usage = sr_object_new();
        sr_object_insert_str(&usage, "organization", usage_records[i].org);
        sr_object_insert_str(&usage, "metric", usage_records[i].metric);
        sr_object_insert_int(&usage, "value", usage_records[i].value);
        sr_object_insert_str(&usage, "period", usage_records[i].period);

        sr_object_t *usage_result = NULL;
        result = sr_insert(db, &err, (sr_value_t **)&usage_result, "usage", &usage);
        CHECK_ERROR(result, err, "Failed to create usage record");
        sr_free_object(usage);
    }
    printf("  [OK] Created 13 usage records (api_calls, storage_mb, active_users)\n");

    // ========================================================================
    // Step 9: Query - Organization with Subscription Details
    // ========================================================================
    print_separator("Step 9: Query - Organizations with Their Plans");

    const char *query1 = "SELECT name, industry, status, "
                         "  ->subscribes_to->plan.name AS plan_name, "
                         "  ->subscribes_to->plan.price_monthly AS monthly_cost "
                         "FROM organization;";

    sr_arr_res_t *q1_results = NULL;
    result = sr_query(db, &err, &q1_results, query1, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q1_results, result);
    sr_free_arr_res_arr(q1_results, result);

    // ========================================================================
    // Step 10: Query - Users and Their Organizations via Graph Traversal
    // ========================================================================
    print_separator("Step 10: Query - Users with Organization Roles (Graph Traversal)");

    const char *query2 = "SELECT name, email, title, "
                         "  ->member_of.role AS org_role, "
                         "  ->member_of->organization.name AS org_name "
                         "FROM user;";

    sr_arr_res_t *q2_results = NULL;
    result = sr_query(db, &err, &q2_results, query2, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q2_results, result);
    sr_free_arr_res_arr(q2_results, result);

    // ========================================================================
    // Step 11: Query - Monthly Revenue Calculation
    // ========================================================================
    print_separator("Step 11: Query - Monthly Revenue Analysis");

    const char *query3 = "SELECT "
                         "  math::sum(->subscribes_to->plan.price_monthly) AS total_mrr, "
                         "  count() AS paying_customers "
                         "FROM organization WHERE status = 'active' GROUP ALL;";

    sr_arr_res_t *q3_results = NULL;
    result = sr_query(db, &err, &q3_results, query3, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q3_results, result);
    sr_free_arr_res_arr(q3_results, result);

    // ========================================================================
    // Step 12: Query - API Usage by Organization for March 2024
    // ========================================================================
    print_separator("Step 12: Query - API Usage Statistics (March 2024)");

    const char *query4 = "SELECT organization, "
                         "  math::sum(value) AS total_api_calls "
                         "FROM usage "
                         "WHERE metric = 'api_calls' AND period = '2024-03' "
                         "GROUP BY organization;";

    sr_arr_res_t *q4_results = NULL;
    result = sr_query(db, &err, &q4_results, query4, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q4_results, result);
    sr_free_arr_res_arr(q4_results, result);

    // ========================================================================
    // Step 13: Query - Find Admins Across All Organizations
    // ========================================================================
    print_separator("Step 13: Query - All Admin Users");

    const char *query5 =
        "SELECT <-member_of<-user.name AS admin_name, "
        "  <-member_of<-user.email AS admin_email, "
        "  name AS organization "
        "FROM organization "
        "WHERE <-member_of.role CONTAINS 'admin' OR <-member_of.role CONTAINS 'owner';";

    sr_arr_res_t *q5_results = NULL;
    result = sr_query(db, &err, &q5_results, query5, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q5_results, result);
    sr_free_arr_res_arr(q5_results, result);

    // ========================================================================
    // Step 14: Upgrade LocalBiz from Free to Starter Plan
    // ========================================================================
    print_separator("Step 14: Upgrade LocalBiz Subscription");

    // First, delete the old subscription relation
    const char *delete_sub = "DELETE subscribes_to WHERE in = organization:localbiz;";
    sr_arr_res_t *del_results = NULL;
    result = sr_query(db, &err, &del_results, delete_sub, NULL);
    CHECK_ERROR(result, err, "Failed to delete old subscription");
    sr_free_arr_res_arr(del_results, result);
    printf("  [OK] Removed old Free plan subscription\n");

    // Create new subscription to Starter plan
    sr_object_t new_sub = sr_object_new();
    sr_object_insert_str(&new_sub, "started_at", "2024-03-15T10:00:00Z");
    sr_object_insert_str(&new_sub, "billing_cycle", "monthly");
    sr_object_insert_str(&new_sub, "status", "active");
    sr_object_insert_str(&new_sub, "upgraded_from", "plan:free");

    sr_value_t *new_sub_result = NULL;
    result = sr_relate(db, &err, &new_sub_result, "organization:localbiz", "subscribes_to",
                       "plan:starter", &new_sub);
    CHECK_ERROR(result, err, "Failed to create new subscription");
    printf("  [OK] LocalBiz upgraded: Free -> Starter ($29/mo)\n");
    if (new_sub_result)
        sr_free_arr(new_sub_result, result);
    sr_free_object(new_sub);

    // Update organization status from trial to active
    sr_object_t status_update = sr_object_new();
    sr_object_insert_str(&status_update, "status", "active");

    sr_value_t *update_result = NULL;
    result = sr_merge(db, &err, &update_result, "organization:localbiz", &status_update);
    CHECK_ERROR(result, err, "Failed to update organization status");
    printf("  [OK] LocalBiz status updated: trial -> active\n");
    if (update_result)
        sr_free_arr(update_result, result);
    sr_free_object(status_update);

    // ========================================================================
    // Step 15: Final Revenue Check After Upgrade
    // ========================================================================
    print_separator("Step 15: Updated Revenue After Upgrade");

    const char *query6 = "SELECT "
                         "  math::sum(->subscribes_to->plan.price_monthly) AS total_mrr, "
                         "  count() AS total_customers "
                         "FROM organization WHERE status = 'active' GROUP ALL;";

    sr_arr_res_t *q6_results = NULL;
    result = sr_query(db, &err, &q6_results, query6, NULL);
    CHECK_ERROR(result, err, "Failed to execute query");
    print_query_results(q6_results, result);
    sr_free_arr_res_arr(q6_results, result);

    // ========================================================================
    // Step 16: Cleanup - Delete all data
    // ========================================================================
    print_separator("Step 16: Cleanup");

    const char *cleanup_queries[] = {
        "DELETE usage;",        "DELETE member_of;", "DELETE subscribes_to;", "DELETE user;",
        "DELETE organization;", "DELETE feature;",   "DELETE plan;",
    };

    for (int i = 0; i < 7; i++) {
        sr_arr_res_t *cleanup_results = NULL;
        result = sr_query(db, &err, &cleanup_results, cleanup_queries[i], NULL);
        CHECK_ERROR(result, err, "Failed to cleanup");
        sr_free_arr_res_arr(cleanup_results, result);
    }
    printf("  [OK] All data cleaned up\n");

    // ========================================================================
    // Step 17: Disconnect
    // ========================================================================
    print_separator("Step 17: Disconnect");

    sr_surreal_disconnect(db);
    printf("  [OK] Disconnected from database\n");

    printf("\n");
    printf("+========================================================================+\n");
    printf("|                    SaaS Example Completed Successfully!                |\n");
    printf("+========================================================================+\n\n");

    return 0;
}
