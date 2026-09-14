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

```c
#include "surrealdb.h"

sr_surreal_t *db;
sr_string_t err;

// "mem://"                   in-memory
// "surrealkv://database.skv" local file
// "ws://localhost:8000"      remote server
if (sr_connect(&err, &db, "mem://") < 0)
{
    printf("failed to connect: %s\n", err);
    sr_string_free(err);
    return 1;
}

sr_use_ns(db, &err, "test");
sr_use_db(db, &err, "test");

sr_surreal_disconnect(db);
```

Calls return a negative status on failure — `SR_ERROR`, `SR_CLOSED` or
`SR_FATAL` — and write a message to `err_ptr` for the caller to release with
`sr_string_free`. Passing `NULL` for `err_ptr` discards the message. Anything a
`sr_*_new` or `sr_value_*` constructor hands back is released by the matching
`sr_*_free`.

Requires SurrealDB 3.2.

## Runtime options

`sr_connect_with_options` takes timeouts and a capability sandbox. Every field's
zero value means "SurrealDB's default", so a zero-initialised `sr_option_t`
behaves exactly like `sr_connect`.

```c
sr_option_t opts = {0};
opts.query_timeout = 30;                              // seconds

// Capabilities are a sandbox: outbound network access is denied by default,
// and scripting is off. Name what you want to change and nothing else.
opts.capabilities.scripting = SR_TOGGLE_ON;

static const char *const allowed[] = { "example.com:443" };
opts.capabilities.allow_network.mode  = SR_TARGET_SOME;
opts.capabilities.allow_network.items = allowed;
opts.capabilities.allow_network.len   = 1;

if (sr_connect_with_options(&err, &db, "mem://", opts) < 0)
{
    printf("failed to connect: %s\n", err);   // e.g. an unparseable target
    sr_string_free(err);
    return 1;
}
```

Each set takes `SR_TARGET_DEFAULT`, `_NONE`, `_SOME` or `_ALL`, with `items`
read only for `_SOME`. Where both an allow and a deny set name the same thing,
deny wins. A name that cannot be parsed is reported as an error rather than
skipped, so a typo cannot quietly widen or narrow the sandbox.

The experimental features are opted into the same way — `"gql"` is required by
the `gql` and `graphql` RPC methods, and `"files"` by `file://` values.

## Sessions

An RPC context (`sr_surreal_rpc_new`) carries a session map. Since SurrealDB 3.1
every request names a session explicitly, so the context mints one for itself
and `sr_surreal_rpc_execute` runs against it.

```c
sr_uuid_t id = {{0}};                       // all-zero: generate an id
sr_rpc_session_attach(rpc, &err, &id);
sr_rpc_execute_on(rpc, &err, &res, &id, request, request_len);
sr_rpc_session_detach(rpc, &err, &id);      // also cancels its live queries
```

`sr_rpc_session_list` reports the active ids (free with `sr_uuid_arr_free`) and
`sr_rpc_session_default` gives the one the context created. Setting
`opts.session_dir` persists sessions to disk, so an attached session outlives
the context that created it.

## Building

```sh
cargo build --release
```

Produces `target/release/libsurrealdb_c.a` and writes `include/surrealdb.h`;

Include the header and link the archive.

```sh
cc app.c -Iinclude target/release/libsurrealdb_c.a -lm -ldl -lpthread     # Linux
```

On macOS, replace the trailing libraries with `-framework Security
-framework SystemConfiguration -framework CoreFoundation -framework IOKit
-lobjc`. Link the archive by path rather than with `-lsurrealdb_c`, which would
otherwise find the shared library built alongside it.

### Testing

```sh
cargo test
```

Runs the whole C test corpus and every example in `c_test/examples`, with no
CMake step. The examples are complete programs compiled and run as tests, so
one that stops building or stops working fails the suite.

### CMake

Not required to build or run anything -- `cargo build` produces the library and
`cargo test` runs the suite. CMake is optional, for running the same corpus
under `ctest` and for installing the header and archive so consumers can
`find_package` them.

```sh
cmake -S . -B build && cmake --build build
cmake --install build --prefix /usr/local
ctest --test-dir build
```

Consumers then get the include path and the platform libraries for free:

```cmake
find_package(surrealdb_c REQUIRED)
target_link_libraries(myapp PRIVATE surrealdb::surrealdb_c)
```

Needs CMake 3.21 or newer and a C23 compiler.
