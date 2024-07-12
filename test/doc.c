#include <stdio.h>
#include <pthread.h>
#include <time.h>
#include <assert.h>
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

    ArrayResultArrayResult res = query(db, "CREATE foo:1 SET val = 42; CREATE foo:1 SET val = 48; SELECT * FROM foo;");
    if (res.err.code != 0)
    {
        printf("%s", res.err.msg);
        free_string(res.err.msg);
        return 1;
    }
    ArrayResultArray arr_res_arr = res.ok;
    assert(arr_res_arr.len == 3);
    assert(arr_res_arr.arr[0].err.code == 0);
    assert(arr_res_arr.arr[1].err.code != 0);
    printf("error: %s\n", arr_res_arr.arr[1].err.msg); // error: Database record `foo:1` already exists
    free_string(arr_res_arr.arr[1].err.msg);
    assert(arr_res_arr.arr[2].err.code == 0);

    array_t foos = arr_res_arr.arr[2].ok;
    double sum = 0;
    for (size_t i = 0; i < foos.len; i++)
    {
        value_t foo = foos.arr[i];
        assert(foo.tag == Object);
        const value_t *val = get(&foo.object, "val");
        switch (val->tag)
        {
        case Number:
            switch (val->number.tag)
            {
            case Int:
                sum += val->number.int_;
                break;
            case Float:
                sum += val->number.float_;
                break;
            default:
                break;
            }
            break;

        default:
            break;
        }
    }
    printf("total of foo vals: %f\n", sum);
    free_arr_res_arr(arr_res_arr);
}