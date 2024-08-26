#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include "../surrealdb.h"

void test_query(sr_surreal_t *db);
void test_select(sr_surreal_t *db);
void *print_stream(void *vargp);

int main()
{
    sr_surreal_t *db;
    sr_string_t err;
    // if (sr_connect(&err, &db, "memory") < 0)
    if (sr_connect(&err, &db, "ws://localhost:8000") < 0)
    {

        printf("%s", err);
        return 1;
    }

    sr_string_t ver;

    if (sr_version(db, &err, &ver) < 0)
    {
        printf("%s", err);
        return 1;
    }
    printf("%s\n", ver);
    sr_free_string(ver);

    if (sr_use_ns(db, &err, "test") < 0)
    {

        printf("%s", err);
        return 1;
    }
    printf("after use ns\n");
    if (sr_use_db(db, &err, "test") < 0)
    {

        printf("%s", err);
        return 1;
    }
    printf("after use db\n");

    sr_arr_res_t *foo_res;
    int len = sr_query(db, &err, &foo_res, "create foo", 0);
    printf("after query\n");
    if (len < 0)
    {
        printf("%s\n", err);
        return 1;
    }
    sr_free_arr_res_arr(foo_res, len);
    printf("after foo query\n");

    sr_stream_t *stream;
    if (sr_select_live(db, &err, &stream, "foo") < 0)
    {
        printf("%s", err);
        return 1;
    }

    len = sr_query(db, &err, &foo_res, "create foo", 0);
    // assert this will work
    sr_free_arr_res_arr(foo_res, len);

    sr_notification_t not ;

    if (sr_stream_next(stream, &not ) > 0)
    {

        sr_print_notification(&not );
    }

    sr_stream_kill(stream);

    // test_select(db);
    test_query(db);

    // printf("%s\n", sel_res);

    // char *res2 = query(db, "select * from foo");
    // printf("%s\n", res2);

    // char *res3 = select(db, "foo");
    // printf("%s\n\n", res3);
}

void test_select(sr_surreal_t *db)
{
    sr_string_t err;
    sr_value_t *foos;
    int len = sr_select(db, &err, &foos, "foo");
    if (len < 0)
    {
        printf("%s", err);
        return;
    }
    sr_value_print(&foos[0]);
    sr_free_arr(foos, len);
}

void test_query(sr_surreal_t *db)
{
    sr_string_t err;
    sr_arr_res_t *res_arr;

    sr_object_t vars = sr_object_new();
    sr_object_insert_int(&vars, "other", 23);

    int len = sr_query(db, &err, &res_arr, "CREATE foo SET val = 42, other = $other; select * from foo;", &vars);
    if (len < 0)
    {
        printf("%s", err);
        return;
    }

    for (size_t i = 0; i < len; i++)
    {
        if (res_arr[i].err.code < 0)
        {
            printf("error for %d: %s\n", (int)i, res_arr[i].err.msg);
            continue;
        }
        sr_array_t arr = res_arr[i].ok;
        for (size_t j = 0; j < arr.len; j++)
        {
            sr_value_t v = arr.arr[j];
            sr_value_print(&v);
        }
    }

    // printf("%s\n\n", res.ok);
}

// void *print_stream(void *vargp)
// {
//     Stream *stream = vargp;
//     Notification *next = stream->next();
// }