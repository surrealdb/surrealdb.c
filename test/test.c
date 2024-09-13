
#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include "../surrealdb.h"

int test_version(sr_surreal_t *db);
int test_query(sr_surreal_t *db);
int test_create_select(sr_surreal_t *db);

int run_test(const char *endpoint, const char *name, int (*test_fun)(sr_surreal_t *))
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
    int res = (*test_fun)(db);
    if (res)
    {
        printf("test: %s failed on %s\n", name, endpoint);
    }
    else
    {
        printf("test: %s succeeded on %s\n", name, endpoint);
    }

    return res;
}

int run_endpoint(const char *endpoint)
{
    int errors = 0;
    errors += run_test(endpoint, "version", &test_version);
    errors += run_test(endpoint, "query", &test_query);
    errors += run_test(endpoint, "create/select", &test_create_select);
    return errors;
}

int main()
{
    int errors = 0;
    srand(42);
    errors += run_endpoint("memory");
    printf("\n");
    errors += run_endpoint("ws://localhost:8000");
    printf("\n");
    // TODO: fix http
    // errors += run_endpoint("http://localhost:8000");
    // printf("\n");

    if (errors == 0)
    {
        printf("succeeded all tests");
        return 0;
    }
    else
    {
        printf("failed with %d errors", errors);
        return 1;
    }
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
    printf("%s", ver);
    sr_free_string(ver);
    return 0;
}

int test_query(sr_surreal_t *db)
{
    sr_string_t err;
    sr_arr_res_t *res_arr;

    sr_object_t vars = sr_object_new();
    sr_object_insert_int(&vars, "val", 23);

    int len = sr_query(db, &err, &res_arr, "CREATE ONLY foo SET val = $val; select value val from only foo limit 1;", &vars);
    if (len < 0)
    {
        printf("%s", err);
        return 1;
    }

    if (len != 2)
    {
        printf("query returned wrong number of items");
        return 1;
    }

    for (size_t i = 0; i < len; i++)
    {
        if (res_arr[i].err.code < 0)
        {
            printf("error for %d: %s\n", (int)i, res_arr[i].err.msg);
            return 1;
        }
    }

    // right length and no errors

    sr_array_t create_res = res_arr[0].ok;

    if (create_res.arr[0].tag != SR_VALUE_OBJECT)
    {
        printf("create should return object");
        return 1;
    }

    sr_object_t create_obj = create_res.arr[0].sr_value_object;
    const sr_value_t *val_field = sr_object_get(&create_obj, "val");

    if (val_field->tag != SR_VALUE_NUMBER)
    {
        printf("field val was created with int so should be number");
        return 1;
    }

    sr_number_t val_field_number = val_field->sr_value_number;

    if (val_field_number.tag != SR_NUMBER_INT)
    {
        printf("field val was created with int so should be int");
        return 1;
    }

    if (val_field_number.sr_number_int != 23)
    {
        printf("field val was created with value 23 but it was %lld instead", val_field_number.sr_number_int);
        return 1;
    }

    sr_array_t select_res = res_arr[1].ok;

    sr_value_t select_expected = {
        .tag = SR_VALUE_NUMBER,
        .sr_value_number = {
            .tag = SR_NUMBER_INT,
            .sr_number_int = 23},
    };

    if (create_res.len != 1)
    {
        printf("create only should have length 1");
        return 1;
    }

    if (!sr_value_eq(&select_res.arr[0], &select_expected))
    {
        printf("create had incorrect value: \n");
        sr_value_print(&select_res.arr[0]);
        printf("but expected:\n");
        sr_value_print(&select_expected);
        return 1;
    }

    sr_free_arr_res_arr(res_arr, len);

    // printf("%s\n\n", res.ok);

    return 0;
}

int test_create_select(sr_surreal_t *db)
{
    sr_string_t err;

    sr_object_t foo1_obj = sr_object_new();
    sr_object_insert_str(&foo1_obj, "val", "hello surreal");
    // sr_value_t foo1_val = {
    //     .tag = SR_VALUE_OBJECT,
    //     .sr_value_object = foo1_obj};

    sr_value_t *foo1_res;
    sr_create(db, err, &foo1_res, "foo:1", &foo1_obj);

    sr_object_t foo2_obj = sr_object_new();
    sr_object_insert_float(&foo2_obj, "val", 4.2);

    // sr_arr_res_t *res_arr;
    // // already checked query so assume this works
    // int len = sr_query(db, &err, &res_arr, "CREATE foo:1 SET val = 2; CREATE foo:2 SET val = 4;", 0);
    // if (len < 0)
    // {
    //     printf("%s", err);
    //     return 1;
    // }
    // sr_free_arr_res_arr(res_arr, len);

    // printf("%s\n\n", res.ok);

    return 0;
}