#include <stdio.h>
#include "../surrealdb.h"

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

    ArrayResult sel_res = select(db, "foo");
    if (sel_res.err.code != 0)
    {
        printf("%s", sel_res.err.msg);
        return 1;
    }
    else
    {
        print_value(&sel_res.ok.arr[0]);
    }
    // printf("%s\n", sel_res);

    ArrayResultArrayResult res = query(db, "create foo; create bar");
    if (res.err.code != 0)
    {
        printf("%s", res.err.msg);
        return 1;
    }
    else
    {
        for (size_t i = 0; i < res.ok.len; i++)
        {
            if (res.ok.arr[i].err.code != 0)
            {
                printf("error: %s", res.err.msg);
                return 1;
            }
        }

        // printf("%s\n\n", res.ok);
    }

    // char *res2 = query(db, "select * from foo");
    // printf("%s\n", res2);

    // char *res3 = select(db, "foo");
    // printf("%s\n\n", res3);
}