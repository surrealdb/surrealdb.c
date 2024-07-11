<br>

<p align="center">
    <img width=120 src="https://raw.githubusercontent.com/surrealdb/icons/main/surreal.svg" />
    &nbsp;
    <img width=120 src="https://raw.githubusercontent.com/surrealdb/icons/main/c.svg" />
</p>

<h3 align="center">The official SurrealDB SDK for C.</h3>

<br>

<p align="center">
    <a href="https://github.com/surrealdb/surrealdb.c"><img src="https://img.shields.io/badge/status-beta-ff00bb.svg?style=flat-square"></a>
    &nbsp;
    <a href="https://surrealdb.com/docs/integration/libraries/c"><img src="https://img.shields.io/badge/docs-view-44cc11.svg?style=flat-square"></a>
    &nbsp;
</p>

<p align="center">
    <a href="https://surrealdb.com/discord"><img src="https://img.shields.io/discord/902568124350599239?label=discord&style=flat-square&color=5a66f6"></a>
    &nbsp;
    <a href="https://twitter.com/surrealdb"><img src="https://img.shields.io/badge/twitter-follow_us-1d9bf0.svg?style=flat-square"></a>
    &nbsp;
    <a href="https://www.linkedin.com/company/surrealdb/"><img src="https://img.shields.io/badge/linkedin-connect_with_us-0a66c2.svg?style=flat-square"></a>
    &nbsp;
    <a href="https://www.youtube.com/channel/UCjf2teVEuYVvvVC-gFZNq6w"><img src="https://img.shields.io/badge/youtube-subscribe-fc1c1c.svg?style=flat-square"></a>
</p>

# surrealdb.c

The official SurrealDB SDK for C.

## Getting started

Connect to an in-memory instance, SurrealKV or remote

```c
#include "path/to/surrealdb.h"

SurrealResult connect_res = connect("memory");
if (connect_res.err.code != 0)
{
    printf("%s", connect_res.err.msg);
    return 1;
}
Surreal *db = connect_res.ok;

use_ns(db, "test");
use_db(db, "test");
```

### Using query

```c
    ArrayResultArrayResult res = query(db, "CREATE foo:1 SET val = 42; CREATE foo:1 SET val = 48; SELECT * FROM foo;");
    if (res.err.code != 0)
    {
        printf("%s", res.err.msg);
        return 1;
    }
    ArrayResultArray arr_res_arr = res.ok;
    assert(arr_res_arr.len == 3);
    assert(arr_res_arr.arr[0].err.code == 0);
    assert(arr_res_arr.arr[1].err.code != 0);
    printf("error: %s\n", arr_res_arr.arr[1].err.msg); // error: Database record `foo:1` already exists
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

```