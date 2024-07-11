#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include "../surrealdb.h"

void test_query(Surreal *db);
void test_select(Surreal *db);
void *print_stream(void *vargp);

Surreal *surreal_cast(void *opaque)
{
    return (Surreal *)opaque;
}

int main()
{
    SurrealResult connect_res = connect("memory");
    if (connect_res.err.code != 0)
    {
        printf("%s", connect_res.err.msg);
        return 1;
    }
    Surreal *db = connect_res.ok;

    StringResult ver_res = version(db);
    if (ver_res.err.code != 0)
    {
        printf("%s", connect_res.err.msg);
        return 1;
    }
    char *ver_str = ver_res.ok;

    printf("%s\n", ver_str);

    use_ns(db, "test");
    use_db(db, "test");

    query(db, "create foo");

    Stream *stream = select_live(db, "foo").ok;

    query(db, "create foo");
    Notification n = next(stream);
    print_notification(&n);

    kill(stream);

    // test_select(db);
    test_query(db);

    // printf("%s\n", sel_res);

    // char *res2 = query(db, "select * from foo");
    // printf("%s\n", res2);

    // char *res3 = select(db, "foo");
    // printf("%s\n\n", res3);
}

void test_select(Surreal *db)
{
    ArrayResult sel_res = select(db, "foo");
    if (sel_res.err.code != 0)
    {
        printf("%s", sel_res.err.msg);
        return;
    }
    else
    {
        print_value(&sel_res.ok.arr[0]);
    }
}

void test_query(Surreal *db)
{
    ArrayResultArrayResult res = query(db, "CREATE foo SET val = 42; select * from foo;");
    if (res.err.code != 0)
    {
        printf("%s", res.err.msg);
        return;
    }
    ArrayResultArray arr_res_arr = res.ok;
    for (size_t i = 0; i < res.ok.len; i++)
    {
        if (res.ok.arr[i].err.code != 0)
        {
            printf("error for %d: %s\n", (int)i, res.ok.arr[i].err.msg);
            continue;
        }
        array_t arr = res.ok.arr[i].ok;
        for (size_t j = 0; j < arr.len; j++)
        {
            value_t v = arr.arr[j];
            print_value(&v);
        }
    }

    // printf("%s\n\n", res.ok);
}

// void *print_stream(void *vargp)
// {
//     Stream *stream = vargp;
//     Notification *next = stream->next();
// }