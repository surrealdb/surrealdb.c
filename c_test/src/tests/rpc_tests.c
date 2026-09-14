#include "unity_fixture.h"
#include "surrealdb.h"
#include "test_support.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(RPC);

TEST_SETUP(RPC) {
}

TEST_TEAR_DOWN(RPC) {
}

TEST(RPC, New) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};
    
    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "rpc_new should succeed: %s", err);
        sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "rpc_new should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "RPC handle should not be NULL");
    
    sr_surreal_rpc_disconnect(rpc);
}

TEST(RPC, Execute) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};
    
    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }
    
    // RPC execute requires CBOR-encoded request bytes
    // For now, test with empty/invalid input to verify it handles errors gracefully
    uint8_t *response = NULL;
    uint8_t invalid_cbor[] = {0x00};  // Invalid CBOR
    
    result = sr_surreal_rpc_execute(rpc, &err, &response, invalid_cbor, 1);
    // This should fail since the CBOR is invalid, but shouldn't crash
    if (result < 0) {
        if (err) sr_string_free(err);
    }
    if (response) {
        sr_byte_arr_free(response, result);
    }
    
    sr_surreal_rpc_disconnect(rpc);
    // Test passes if we get here without crashing
}

TEST(RPC, Notifications) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err = NULL;
    sr_option_t opts = {0};
    
    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }
    
    sr_RpcStream *stream = NULL;
    result = sr_surreal_rpc_notifications(rpc, &err, &stream);
    if (result < 0) {
        if (err) sr_string_free(err);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("Failed to get notifications stream");
    }
    TEST_ASSERT_NOT_NULL_MESSAGE(stream, "Notifications stream should not be NULL");
    
    sr_rpc_stream_free(stream);
    sr_surreal_rpc_disconnect(rpc);
}

/* Context for the blocked reader below. */
typedef struct {
    sr_RpcStream *stream;
    int rc;
} rpc_reader_ctx;

static void rpc_reader(void *arg) {
    rpc_reader_ctx *ctx = (rpc_reader_ctx *)arg;
    uint8_t *payload = NULL;

    /* Blocks: nothing is pending, and there is no timeout variant. */
    ctx->rc = sr_rpc_stream_next(ctx->stream, &payload);
    if (ctx->rc > 0 && payload != NULL) {
        sr_byte_arr_free(payload, ctx->rc);
    }
}

TEST(RPC, StreamNextUnblocksOnShutdown) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err = NULL;
    sr_option_t opts = {0};

    if (sr_surreal_rpc_new(&err, &rpc, "memory", opts) < 0) {
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }

    sr_RpcStream *stream = NULL;
    if (sr_surreal_rpc_notifications(rpc, &err, &stream) < 0) {
        if (err) sr_string_free(err);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("Failed to get notifications stream");
    }

    /*
     * The contract under test is how a consumer shuts down: a worker parked in
     * sr_rpc_stream_next is released by freeing the connection, which drops the
     * sending half of the notification channel. Without that there would be no
     * way to retire the thread short of ending the process.
     */
    rpc_reader_ctx ctx = {stream, 0};
    test_thread *reader = test_thread_start(rpc_reader, &ctx);
    TEST_ASSERT_NOT_NULL_MESSAGE(reader, "could not start the reader thread");

    test_sleep_ms(300);
    sr_surreal_rpc_disconnect(rpc);
    test_thread_join(reader);

    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_CLOSED, ctx.rc,
                                  "closing the connection must release the reader");

    /* The stream outlives its connection and is still ours to free. */
    sr_rpc_stream_free(stream);
}

/* A hand-rolled CBOR map: {"method": <name>, "params": [<arg>]}. Encoding this
   by hand keeps the test suite free of a CBOR dependency; the request shape is
   small enough that doing so stays readable. */
static int rpc_request_1(uint8_t *buf, int cap, const char *method, const char *arg) {
    const int mlen = (int)strlen(method), alen = (int)strlen(arg);
    if (mlen > 23 || alen > 255 || cap < 32 + mlen + alen) return -1;
    int i = 0;
    buf[i++] = 0xA2;                                   /* map(2) */
    buf[i++] = 0x66; memcpy(buf + i, "method", 6); i += 6;
    buf[i++] = (uint8_t)(0x60 | mlen); memcpy(buf + i, method, (size_t)mlen); i += mlen;
    buf[i++] = 0x66; memcpy(buf + i, "params", 6); i += 6;
    buf[i++] = 0x81;                                   /* array(1) */
    if (alen < 24) { buf[i++] = (uint8_t)(0x60 | alen); }
    else { buf[i++] = 0x78; buf[i++] = (uint8_t)alen; }
    memcpy(buf + i, arg, (size_t)alen); i += alen;
    return i;
}

/* As above, with two string params -- `use` takes (namespace, database). */
static int rpc_request_2(uint8_t *buf, int cap, const char *method,
                         const char *a, const char *b) {
    const int mlen = (int)strlen(method);
    const int alen = (int)strlen(a), blen = (int)strlen(b);
    if (mlen > 23 || alen > 23 || blen > 23 || cap < 32 + mlen + alen + blen) return -1;
    int i = 0;
    buf[i++] = 0xA2;                                   /* map(2) */
    buf[i++] = 0x66; memcpy(buf + i, "method", 6); i += 6;
    buf[i++] = (uint8_t)(0x60 | mlen); memcpy(buf + i, method, (size_t)mlen); i += mlen;
    buf[i++] = 0x66; memcpy(buf + i, "params", 6); i += 6;
    buf[i++] = 0x82;                                   /* array(2) */
    buf[i++] = (uint8_t)(0x60 | alen); memcpy(buf + i, a, (size_t)alen); i += alen;
    buf[i++] = (uint8_t)(0x60 | blen); memcpy(buf + i, b, (size_t)blen); i += blen;
    return i;
}

/* Run one CBOR request on `session`, failing the test if it errors. */
static void rpc_run_on(sr_surreal_rpc_t *rpc, const sr_uuid_t *session,
                       const uint8_t *req, int len, const char *what) {
    sr_string_t err = NULL;
    uint8_t *res = NULL;
    int rc = sr_rpc_execute_on(rpc, &err, &res, session, req, len);
    if (rc < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "%s should succeed: %s", what,
                 err ? (const char *)err : "(no error)");
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE(msg);
    }
    if (res) sr_byte_arr_free(res, rc);
    if (err) sr_string_free(err);
}

/*
 * `query` is the method the whole RPC surface exists for, and it was
 * unreachable: run_rpc matched only DbResult::Other, so `query` and `gql` --
 * the two methods that return DbResult::Query -- came back as
 * "RPC::execute had unimplemented response". Nothing caught it because the
 * other tests only drive methods that return Other (ping, info, version, use).
 */
TEST(RPC, QueryIsReachable) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err = NULL;
    sr_option_t opts = {0};

    if (sr_surreal_rpc_new(&err, &rpc, "memory", opts) < 0) {
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }

    uint8_t req[128];
    int n = rpc_request_1(req, (int)sizeof(req), "query", "RETURN 1;");
    TEST_ASSERT_GREATER_THAN_INT(0, n);

    uint8_t *res = NULL;
    int rc = sr_surreal_rpc_execute(rpc, &err, &res, req, n);

    if (rc < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "query over RPC should succeed: %s",
                 err ? (const char *)err : "(no error)");
        if (err) { sr_string_free(err); err = NULL; }
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE(msg);
    }

    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, rc, "a query response should carry bytes");
    TEST_ASSERT_NOT_NULL_MESSAGE(res, "a query response should be written");

    sr_byte_arr_free(res, rc);
    if (err) sr_string_free(err);
    sr_surreal_rpc_disconnect(rpc);
}

/*
 * Detaching a session must stop the live queries it registered.
 *
 * Core calls `cleanup_lqs` from `del_session` for exactly this, but the hook is
 * the transport's to implement and ours was an empty body, so a detached
 * session left its live queries registered in the datastore -- still pushing
 * into the shared notification channel with no owner and no way to identify
 * them. The same hook is called on signin, signup, authenticate, refresh,
 * invalidate and reset, so the leak was not limited to teardown: a live query
 * survived the re-authentication that was supposed to retire it.
 *
 * The pre-detach half is the control. Without it a broken pipeline would pass
 * the post-detach assertion for the wrong reason.
 */
TEST(RPC, DetachCancelsLiveQueries) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err = NULL;
    sr_option_t opts = {0};

    if (sr_surreal_rpc_new(&err, &rpc, "memory", opts) < 0) {
        if (err) sr_string_free(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }

    sr_RpcStream *stream = NULL;
    if (sr_surreal_rpc_notifications(rpc, &err, &stream) < 0) {
        if (err) sr_string_free(err);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("notifications stream should open");
    }

    uint8_t req[256];
    int n;

    sr_uuid_t owner;
    if (sr_rpc_session_attach(rpc, &err, &owner) < 0) {
        if (err) sr_string_free(err);
        sr_rpc_stream_free(stream);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("session attach should succeed");
    }

    n = rpc_request_2(req, (int)sizeof(req), "use", "test", "test");
    TEST_ASSERT_GREATER_THAN_INT(0, n);
    rpc_run_on(rpc, &owner, req, n, "use on the owning session");

    n = rpc_request_1(req, (int)sizeof(req), "query",
                      "DEFINE TABLE lqt SCHEMALESS; LIVE SELECT * FROM lqt;");
    TEST_ASSERT_GREATER_THAN_INT(0, n);
    rpc_run_on(rpc, &owner, req, n, "registering a live query");

    /* Control: the live query is live, so a write reaches the channel. */
    n = rpc_request_1(req, (int)sizeof(req), "query", "CREATE lqt:1 SET v = 1;");
    TEST_ASSERT_GREATER_THAN_INT(0, n);
    rpc_run_on(rpc, &owner, req, n, "write before detach");

    uint8_t *payload = NULL;
    int got = sr_rpc_stream_next_timeout(stream, &payload, 2000);
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(
        0, got, "a live query should notify before its session is detached");
    if (payload) sr_byte_arr_free(payload, got);

    /* Detach. This is what must cancel the live query. */
    if (sr_rpc_session_detach(rpc, &err, &owner) < 0) {
        if (err) sr_string_free(err);
        sr_rpc_stream_free(stream);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("session detach should succeed");
    }

    /* Write again, from a session that still exists. */
    sr_uuid_t fallback;
    if (sr_rpc_session_default(rpc, &err, &fallback) < 0) {
        if (err) sr_string_free(err);
        sr_rpc_stream_free(stream);
        sr_surreal_rpc_disconnect(rpc);
        TEST_FAIL_MESSAGE("default session should be available");
    }

    n = rpc_request_2(req, (int)sizeof(req), "use", "test", "test");
    TEST_ASSERT_GREATER_THAN_INT(0, n);
    rpc_run_on(rpc, &fallback, req, n, "use on the fallback session");

    n = rpc_request_1(req, (int)sizeof(req), "query", "CREATE lqt:2 SET v = 2;");
    TEST_ASSERT_GREATER_THAN_INT(0, n);
    rpc_run_on(rpc, &fallback, req, n, "write after detach");

    /*
     * Cancelling a live query emits one final notification, action KILLED with
     * no result, which is how a consumer learns its live query was retired.
     * Expect exactly that, and then silence.
     *
     * The payload is CBOR and there is no decoder here, so the action is
     * checked by looking for the string in the encoded bytes -- CBOR stores
     * text as raw UTF-8, so this is sound if crude.
     */
    payload = NULL;
    got = sr_rpc_stream_next_timeout(stream, &payload, 2000);
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(
        0, got, "cancelling a live query should report it as KILLED");
    TEST_ASSERT_NOT_NULL(payload);
    int saw_killed = 0;
    for (int i = 0; i + 6 <= got; i++) {
        if (memcmp(payload + i, "KILLED", 6) == 0) { saw_killed = 1; break; }
    }
    sr_byte_arr_free(payload, got);
    TEST_ASSERT_TRUE_MESSAGE(saw_killed,
        "the notification after detach should be the KILLED marker");

    /*
     * And now nothing, which is the actual point: the live query is gone from
     * the datastore rather than merely unowned. This assertion is only
     * writable because of sr_rpc_stream_next_timeout -- proving a notification
     * does NOT arrive needs a bounded wait, and the blocking call would park
     * here forever on a correct build.
     */
    payload = NULL;
    got = sr_rpc_stream_next_timeout(stream, &payload, 500);
    if (payload) sr_byte_arr_free(payload, got);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        SR_TIMEOUT, got,
        "a detached session's live query must not keep notifying");

    if (err) sr_string_free(err);
    sr_rpc_stream_free(stream);
    sr_surreal_rpc_disconnect(rpc);
}

TEST(RPC, Free) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};
    
    sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    sr_surreal_rpc_disconnect(rpc);
    // If we get here without crashing, test passes
}

TEST_GROUP_RUNNER(RPC) {
    RUN_TEST_CASE(RPC, New);
    RUN_TEST_CASE(RPC, Execute);
    RUN_TEST_CASE(RPC, Notifications);
    RUN_TEST_CASE(RPC, StreamNextUnblocksOnShutdown);
    RUN_TEST_CASE(RPC, QueryIsReachable);
    RUN_TEST_CASE(RPC, DetachCancelsLiveQueries);
    RUN_TEST_CASE(RPC, Free);
}
