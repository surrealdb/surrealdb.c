/*
 * Session coverage for the RPC context.
 *
 * SurrealDB 3.1 removed the type-level "default session" as part of the fix for
 * GHSA-4vgr-h27g-cf9p: every request names a session explicitly. The C context
 * mints one for itself so the simple execute path still works, and these tests
 * cover the rest of that model -- attach, detach, reset, list, and running a
 * request against a session the caller chose.
 */

#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Session);

static sr_surreal_rpc_t *rpc;
static sr_string_t err;

TEST_SETUP(Session) {
    rpc = NULL;
    err = NULL;
    sr_option_t opts = {0};
    /* gql/graphql are gated twice: by the `gql` Cargo feature at compile time
       and by a runtime capability here. Capabilities are a sandbox, so nothing
       is on by default -- the tests below opt in explicitly. */
    static const char *const experimental[] = { "gql", "files" };
    opts.capabilities.allow_experimental.mode = SR_TARGET_SOME;
    opts.capabilities.allow_experimental.items = experimental;
    opts.capabilities.allow_experimental.len = 2;
    if (sr_surreal_rpc_new(&err, &rpc, "memory", opts) < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        rpc = NULL;
    }
}

TEST_TEAR_DOWN(Session) {
    if (err) { sr_string_free(err); err = NULL; }
    if (rpc) { sr_surreal_rpc_disconnect(rpc); rpc = NULL; }
}

static const sr_uuid_t ZERO_UUID = {{0}};

static int uuid_eq(const sr_uuid_t *a, const sr_uuid_t *b) {
    return memcmp(a->_0, b->_0, 16) == 0;
}

static int list_contains(const sr_uuid_t *list, int n, const sr_uuid_t *want) {
    for (int i = 0; i < n; ++i) if (uuid_eq(&list[i], want)) return 1;
    return 0;
}

/* A hand-rolled CBOR map: {"method": <name>, "params": []}. Encoding this by
   hand keeps the test suite free of a CBOR dependency, and the request shape is
   small enough that doing so stays readable. */
static int build_request(uint8_t *buf, int cap, const char *method) {
    const int mlen = (int)strlen(method);
    if (mlen > 23 || cap < 1 + 1 + 6 + 1 + mlen + 1 + 6 + 1) return -1;
    int i = 0;
    buf[i++] = 0xA2;                            /* map(2) */
    buf[i++] = 0x66; memcpy(buf + i, "method", 6); i += 6;
    buf[i++] = (uint8_t)(0x60 | mlen); memcpy(buf + i, method, (size_t)mlen); i += mlen;
    buf[i++] = 0x66; memcpy(buf + i, "params", 6); i += 6;
    buf[i++] = 0x80;                            /* array(0) */
    return i;
}

/* {"method": <name>, "params": [<arg>]} */
static int build_request_1(uint8_t *buf, int cap, const char *method, const char *arg) {
    const int mlen = (int)strlen(method), alen = (int)strlen(arg);
    if (mlen > 23 || alen > 255 || cap < 32 + mlen + alen) return -1;
    int i = 0;
    buf[i++] = 0xA2;
    buf[i++] = 0x66; memcpy(buf + i, "method", 6); i += 6;
    buf[i++] = (uint8_t)(0x60 | mlen); memcpy(buf + i, method, (size_t)mlen); i += mlen;
    buf[i++] = 0x66; memcpy(buf + i, "params", 6); i += 6;
    buf[i++] = 0x81;                                   /* array(1) */
    if (alen < 24) { buf[i++] = (uint8_t)(0x60 | alen); }
    else { buf[i++] = 0x78; buf[i++] = (uint8_t)alen; }
    memcpy(buf + i, arg, (size_t)alen); i += alen;
    return i;
}

/* Point a session at a namespace and database. */
static int session_use(const sr_uuid_t *id, sr_string_t *e) {
    uint8_t r[64];
    int i = 0;
    r[i++] = 0xA2;
    r[i++] = 0x66; memcpy(r + i, "method", 6); i += 6;
    r[i++] = 0x63; memcpy(r + i, "use", 3); i += 3;
    r[i++] = 0x66; memcpy(r + i, "params", 6); i += 6;
    r[i++] = 0x82;
    r[i++] = 0x62; memcpy(r + i, "ns", 2); i += 2;
    r[i++] = 0x62; memcpy(r + i, "db", 2); i += 2;
    uint8_t *out = NULL;
    int n = sr_rpc_execute_on(rpc, e, &out, id, r, i);
    if (n > 0 && out) sr_byte_arr_free(out, n);
    return n;
}

TEST(Session, GqlIsTheGraphQueryLanguage) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* `gql` is SurrealDB's MATCH-based graph query language, NOT GraphQL --
       `graphql` is the separate method for that. Easy to conflate given the
       names, so this pins the distinction down.

       Both the Cargo feature and the runtime capability must be on; with only
       the feature you get "Experimental capability `gql` is not enabled" at the
       point of use, which is a confusing place to find out. */
    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));
    session_use(&id, &err);
    if (err) { sr_string_free(err); err = NULL; }

    uint8_t req[256];
    int n = build_request_1(req, (int)sizeof(req), "gql", "RETURN 1");
    TEST_ASSERT_GREATER_THAN_INT(0, n);

    uint8_t *res = NULL;
    int rc = sr_rpc_execute_on(rpc, &err, &res, &id, req, n);

    /* A MATCH-less query is rejected by the GQL parser -- which is the proof
       that it reached that parser rather than being refused as unavailable. */
    TEST_ASSERT_LESS_THAN_INT_MESSAGE(0, rc, "a MATCH-less gql query should be rejected");
    TEST_ASSERT_NOT_NULL(err);
    if (err) {
        TEST_ASSERT_NULL_MESSAGE(strstr(err, "not enabled"),
                                 "the capability must be on, so this is a parse error");
        sr_string_free(err); err = NULL;
    }
    if (rc > 0 && res) sr_byte_arr_free(res, rc);
}

TEST(Session, GraphqlDispatchesToTheEngine) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* GraphQL reaches its engine, which then reports that the *database* has no
       GraphQL configuration -- a DDL step (DEFINE CONFIG GRAPHQL), not a gap in
       the C API. What matters here is that the method is reachable at all. */
    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));
    session_use(&id, &err);
    if (err) { sr_string_free(err); err = NULL; }

    uint8_t req[256];
    int n = build_request_1(req, (int)sizeof(req), "graphql", "{ __typename }");
    uint8_t *res = NULL;
    int rc = sr_rpc_execute_on(rpc, &err, &res, &id, req, n);

    if (rc < 0) {
        TEST_ASSERT_NOT_NULL(err);
        TEST_ASSERT_NULL_MESSAGE(strstr(err, "not found"),
                                 "graphql must dispatch, not be unrecognised");
        sr_string_free(err); err = NULL;
    } else if (res) {
        sr_byte_arr_free(res, rc);
    }
}

TEST(Session, DefaultExists) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t def = ZERO_UUID;
    int rc = sr_rpc_session_default(rpc, &err, &def);
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, rc, "default session should be readable");
    TEST_ASSERT_FALSE_MESSAGE(uuid_eq(&def, &ZERO_UUID),
                              "the context must mint a real default session id");

    /* And it must be in the list, because sr_surreal_rpc_execute runs on it. */
    sr_uuid_t *list = NULL;
    int n = sr_rpc_session_list(rpc, &err, &list);
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, n, "at least the default session exists");
    TEST_ASSERT_TRUE_MESSAGE(list_contains(list, n, &def),
                             "the default session must appear in the session list");
    sr_uuid_arr_free(list, n);
}

TEST(Session, AttachGeneratesAnId) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t id = ZERO_UUID;
    int rc = sr_rpc_session_attach(rpc, &err, &id);
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, rc, "attach should succeed");
    TEST_ASSERT_FALSE_MESSAGE(uuid_eq(&id, &ZERO_UUID),
                              "an all-zero id must be replaced with a generated one");

    sr_uuid_t *list = NULL;
    int n = sr_rpc_session_list(rpc, &err, &list);
    TEST_ASSERT_TRUE_MESSAGE(list_contains(list, n, &id), "attached session should be listed");
    sr_uuid_arr_free(list, n);
}

TEST(Session, AttachHonoursACallerId) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t wanted = {{0xAB, 0xCD, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14}};
    sr_uuid_t id = wanted;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));
    TEST_ASSERT_TRUE_MESSAGE(uuid_eq(&id, &wanted), "a supplied id must be used as given");

    /* Attaching the same id twice is an error, not a silent overwrite. */
    sr_uuid_t again = wanted;
    int rc = sr_rpc_session_attach(rpc, &err, &again);
    TEST_ASSERT_LESS_THAN_INT_MESSAGE(0, rc, "attaching a duplicate id must fail");
    if (err) { sr_string_free(err); err = NULL; }
}

TEST(Session, DetachRemovesIt) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));

    sr_uuid_t *before = NULL;
    int nb = sr_rpc_session_list(rpc, &err, &before);
    sr_uuid_arr_free(before, nb);

    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, sr_rpc_session_detach(rpc, &err, &id),
                                             "detach should succeed");

    sr_uuid_t *after = NULL;
    int na = sr_rpc_session_list(rpc, &err, &after);
    TEST_ASSERT_EQUAL_INT_MESSAGE(nb - 1, na, "detach should remove exactly one session");
    TEST_ASSERT_FALSE_MESSAGE(list_contains(after, na, &id),
                              "a detached session must not be listed");
    sr_uuid_arr_free(after, na);
}

TEST(Session, ResetKeepsTheSession) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, sr_rpc_session_reset(rpc, &err, &id),
                                             "reset should succeed");

    /* Reset returns the session to its initial state; it does not close it. */
    sr_uuid_t *list = NULL;
    int n = sr_rpc_session_list(rpc, &err, &list);
    TEST_ASSERT_TRUE_MESSAGE(list_contains(list, n, &id),
                             "reset must not remove the session");
    sr_uuid_arr_free(list, n);
}

TEST(Session, ExecuteOnAChosenSession) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));

    uint8_t req[64];
    int reqlen = build_request(req, (int)sizeof(req), "version");
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, reqlen, "request should encode");

    uint8_t *res = NULL;
    int n = sr_rpc_execute_on(rpc, &err, &res, &id, req, reqlen);
    if (n < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "execute_on should succeed: %s", err ? err : "unknown");
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, n, "a version reply should have bytes");
    sr_byte_arr_free(res, n);
}

TEST(Session, StatelessMethodsIgnoreTheSession) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* Not every method consults the session. `version` dispatches as
       `self.version(txn, params)` -- the session id is never passed to it -- so
       it succeeds against an id that was never attached. That is correct, and
       worth pinning down so the next person does not read it as a validation
       hole. Session binding is enforced where a session is actually used; see
       the next test. */
    sr_uuid_t never_attached = {{0xFF, 0xEE, 0xDD, 0xCC, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9}};
    uint8_t req[64];
    int reqlen = build_request(req, (int)sizeof(req), "version");

    uint8_t *res = NULL;
    int n = sr_rpc_execute_on(rpc, &err, &res, &never_attached, req, reqlen);
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, n, "version is stateless and should succeed");
    if (n > 0 && res) sr_byte_arr_free(res, n);
    if (err) { sr_string_free(err); err = NULL; }
}

TEST(Session, StatefulMethodsRejectAnUnknownSession) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* `info` dispatches as `self.info(txn, session)`, so it resolves the
       session through get_session() and must refuse an id that was never
       attached. This is the guarantee the 3.1 session rework exists to provide
       (GHSA-4vgr-h27g-cf9p): a request cannot run unbound. */
    sr_uuid_t never_attached = {{0xFF, 0xEE, 0xDD, 0xCC, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9}};
    uint8_t req[64];
    int reqlen = build_request(req, (int)sizeof(req), "info");

    uint8_t *res = NULL;
    int n = sr_rpc_execute_on(rpc, &err, &res, &never_attached, req, reqlen);
    if (n >= 0) {
        if (n > 0 && res) sr_byte_arr_free(res, n);
        TEST_FAIL_MESSAGE("a stateful method must not run on an unattached session");
    }
    TEST_ASSERT_NOT_NULL_MESSAGE(err, "the rejection should carry a message");
    if (err) { sr_string_free(err); err = NULL; }
}

TEST(Session, NullArgumentsAreRejected) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* Null out-params are a caller error and must be reported, not crash. */
    TEST_ASSERT_LESS_THAN_INT(0, sr_rpc_session_attach(rpc, &err, NULL));
    if (err) { sr_string_free(err); err = NULL; }
    TEST_ASSERT_LESS_THAN_INT(0, sr_rpc_session_detach(rpc, &err, NULL));
    if (err) { sr_string_free(err); err = NULL; }
    TEST_ASSERT_LESS_THAN_INT(0, sr_rpc_session_reset(rpc, &err, NULL));
    if (err) { sr_string_free(err); err = NULL; }
    TEST_ASSERT_LESS_THAN_INT(0, sr_rpc_session_list(rpc, &err, NULL));
    if (err) { sr_string_free(err); err = NULL; }

    /* And a null err_ptr is documented as legal on all of them. */
    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, NULL, &id));
    TEST_ASSERT_LESS_THAN_INT(0, sr_rpc_session_attach(rpc, NULL, &id)); /* duplicate */
}

TEST(Session, ProtocolMethodsWithNoDirectEquivalent) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* Six protocol methods have no direct sr_* function, because the Rust SDK
       that the direct path is built on does not expose them: ping, info,
       refresh, revoke, gql and graphql. They are still reachable by name
       through the RPC path, which is what this checks -- each one must
       *dispatch*, i.e. come back as a result or a real error rather than being
       parsed as Method::Unknown or crashing.

       gql/graphql are omitted: the `gql` feature is not enabled on
       surrealdb-core in this build, so they would fail for a reason that has
       nothing to do with dispatch. */
    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));

    static const char *const methods[] = { "ping", "info", "refresh", "revoke" };
    for (unsigned i = 0; i < sizeof(methods) / sizeof(methods[0]); ++i) {
        uint8_t req[64];
        int reqlen = build_request(req, (int)sizeof(req), methods[i]);
        TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, reqlen, "request should encode");

        uint8_t *res = NULL;
        int n = sr_rpc_execute_on(rpc, &err, &res, &id, req, reqlen);

        /* Either outcome is acceptable -- refresh and revoke legitimately fail
           without an authenticated session. What must not happen is a crash, or
           an error naming the method as unrecognised. */
        if (n < 0) {
            TEST_ASSERT_NOT_NULL_MESSAGE(err, "a failure should carry a message");
            if (err) {
                TEST_ASSERT_NULL_MESSAGE(strstr(err, "not found"),
                                         "the method must dispatch, not be unrecognised");
                sr_string_free(err);
                err = NULL;
            }
        } else if (res) {
            sr_byte_arr_free(res, n);
        }
    }
}

TEST(Session, PingSucceeds) {
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "rpc context should be created");

    /* Ping is the cheapest liveness check in the protocol and has no direct
       equivalent -- sr_health performs a real datastore round trip instead. */
    sr_uuid_t id = ZERO_UUID;
    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_rpc_session_attach(rpc, &err, &id));

    uint8_t req[64];
    int reqlen = build_request(req, (int)sizeof(req), "ping");
    uint8_t *res = NULL;
    int n = sr_rpc_execute_on(rpc, &err, &res, &id, req, reqlen);
    if (n < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "ping should succeed: %s", err ? err : "unknown");
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE(msg);
    }
    if (res) sr_byte_arr_free(res, n);
}

TEST_GROUP_RUNNER(Session) {
    RUN_TEST_CASE(Session, DefaultExists);
    RUN_TEST_CASE(Session, AttachGeneratesAnId);
    RUN_TEST_CASE(Session, AttachHonoursACallerId);
    RUN_TEST_CASE(Session, DetachRemovesIt);
    RUN_TEST_CASE(Session, ResetKeepsTheSession);
    RUN_TEST_CASE(Session, ExecuteOnAChosenSession);
    RUN_TEST_CASE(Session, StatelessMethodsIgnoreTheSession);
    RUN_TEST_CASE(Session, StatefulMethodsRejectAnUnknownSession);
    RUN_TEST_CASE(Session, ProtocolMethodsWithNoDirectEquivalent);
    RUN_TEST_CASE(Session, PingSucceeds);
    RUN_TEST_CASE(Session, GqlIsTheGraphQueryLanguage);
    RUN_TEST_CASE(Session, GraphqlDispatchesToTheEngine);
    RUN_TEST_CASE(Session, NullArgumentsAreRejected);
}
