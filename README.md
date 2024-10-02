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

> [!WARNING]
> There is an issue with cbindgen which causes incorrect ordering of header files, so linking may fail.\
> This should be fixed soon, or can be worked around by using a published header file or manually reordering
> see: https://github.com/mozilla/cbindgen/issues/981



Connect to an in-memory instance, SurrealKV or remote

```c
#include "path/to/surrealdb.h"

sr_surreal_t *db;
sr_string_t err;

// connect to server
char *endpoint = "ws://localhost:8000";
// connect to file
char *endpoint = "surrealkv://database.skv";

if (sr_connect(&err, &db, endpoint) < 0)
{
    printf("failed to connect: %s", err);
    return 1;
}

sr_surreal_disconnect(db);
```
