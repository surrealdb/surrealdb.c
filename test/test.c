
#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include "../surrealdb.h"

int test_version(sr_surreal_t *db);
int test_query(sr_surreal_t *db);
int test_select(sr_surreal_t *db);

int run_test_inner(const char *endpoint, int (*test_fun)(sr_surreal_t *))
{
    char db_name[16];
    // SAFETY: max int is 10 digits
    sprintf(db_name, "%d", rand());
    sr_surreal_t *db;
    sr_string_t err;
    if (sr_connect(&err, &db, endpoint) < 0)
    {

        printf("%s\n", err);
        return 1;
    }
    if (sr_use_ns(db, &err, "clib_test") < 0)
    {

        printf("%s\n", err);
        return 1;
    }
    if (sr_use_db(db, &err, db_name) < 0)
    {

        printf("%s\n", err);
        return 1;
    }
    return (*test_fun)(db);
}

void run_test(const char *endpoint, const char *name, int (*test_fun)(sr_surreal_t *))
{
    if (run_test_inner(endpoint, test_fun) == 1)
    {
        printf("test: %s failed on %s\n", name, endpoint);
    }
    else
    {
        printf("test: %s succeeded on %s\n", name, endpoint);
    }
}

void run_endpoint(const char *endpoint)
{
    run_test(endpoint, "version", &test_version);
    // run_test(endpoint, "query", &test_query);
    // run_test(endpoint, "select", &test_select);
}

int main()
{
    srand(42);
    run_endpoint("memory");
    run_endpoint("ws://localhost:8000");
    run_endpoint("http://localhost:8000");
};

int test_version(sr_surreal_t *db)
{
    sr_string_t err;
    sr_string_t ver;
    if (sr_version(db, &err, &ver) < 0)
    {
        printf("%s", err);
        return 1;
    }
    return 0;
}