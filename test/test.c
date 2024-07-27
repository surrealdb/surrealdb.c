#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include "../surrealdb.h"

void test_query(sr_surreal_t *db);
void test_select(sr_surreal_t *db);
void *print_stream(void *vargp);

int main()
{
    sr_surreal_res_t connect_res = sr_connect("memory");
    if (connect_res.err.code != 0)
    {
        printf("%s", connect_res.err.msg);
        return 1;
    }
    sr_surreal_t *db = &connect_res.ok;

    sr_string_t ver = sr_version(db);
    if (db->err != 0)
    {
        printf("%s", db->err);
        return 1;
    }

    printf("%s\n", ver);

    sr_use_ns(db, "test");
    sr_use_db(db, "test");

    sr_query(db, "create foo");

    sr_stream_t *stream = sr_select_live(db, "foo");
    if (db->err != 0)
    {
        printf("%s", db->err);
        return 1;
    }

    sr_query(db, "create foo");
    sr_notification_t n = sr_stream_next(stream);
    sr_print_notification(&n);

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
    sr_array_t foos = sr_select(db, "foo");
    if (db->err != 0)
    {
        printf("%s", db->err);
        return;
    }
    else
    {
        sr_value_print(&foos.arr[0]);
    }
}

void test_query(sr_surreal_t *db)
{
    sr_arr_res_arr_t arr_res = sr_query(db, "CREATE foo SET val = 42; select * from foo;");
    if (db->err != 0)
    {
        printf("%s", db->err);
        return;
    }
    // ArrayResultArray arr_res_arr = res.ok;
    for (size_t i = 0; i < arr_res.len; i++)
    {
        if (arr_res.arr[i].err.code != 0)
        {
            printf("error for %d: %s\n", (int)i, arr_res.arr[i].err.msg);
            continue;
        }
        sr_array_t arr = arr_res.arr[i].ok;
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