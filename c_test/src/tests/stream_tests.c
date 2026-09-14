#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(Stream);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Stream) {
    db = NULL;
    sr_connect(&err, &db, "mem://");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(Stream) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

/* Open a live query on a table that exists, or NULL if that is not possible. */
static sr_stream_t *live_on(const char *table) {
    char stmt[128];
    snprintf(stmt, sizeof(stmt), "DEFINE TABLE %s SCHEMALESS", table);

    sr_arr_res_t *res = NULL;
    int n = sr_query(db, &err, &res, stmt, NULL);
    if (n > 0) sr_arr_res_arr_free(res, n);
    if (n < 0 && err) { sr_string_free(err); err = NULL; }

    sr_stream_t *stream = NULL;
    if (sr_select_live(db, &err, &stream, table) < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        return NULL;
    }
    return stream;
}

TEST(Stream, Next) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_stream_t *stream = live_on("streamable");
    if (stream == NULL) {
        TEST_IGNORE_MESSAGE("live queries unavailable on this build");
    }

    sr_arr_res_t *res = NULL;
    int n = sr_query(db, &err, &res, "CREATE streamable:1 SET v = 1", NULL);
    if (n > 0) sr_arr_res_arr_free(res, n);
    if (n < 0 && err) { sr_string_free(err); err = NULL; }

    /*
     * Bounded rather than blocking. The CREATE above should already have queued
     * a notification, but asserting that with sr_stream_next means a build that
     * fails to deliver it hangs the suite instead of failing it -- which is
     * exactly the wrong behaviour to hand to CI.
     */
    sr_notification_t notification;
    int got = sr_stream_next_timeout(stream, &notification, 2000);

    TEST_ASSERT_NOT_EQUAL_INT_MESSAGE(
        SR_TIMEOUT, got, "a notification should arrive within two seconds");

    /* Only a positive result carries a notification. SR_NONE (0) means the stream
       ended without delivering the CREATE above, and a negative result is an error
       or a closed stream; both are failures here. */
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, got, "a notification should be received");

    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_ACTION_CREATE, notification.action,
                                  "a CREATE should be reported as such");
    /* The notification owns its data. sr_value_free must not be used on
       notification.data: that reclaims a Box, and this value lives in our
       own storage. */
    sr_notification_free(notification);

    sr_stream_kill(stream);
}

TEST(Stream, Kill) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_stream_t *stream = live_on("killable_stream");
    if (stream == NULL) {
        TEST_IGNORE_MESSAGE("live queries unavailable on this build");
    }
    sr_stream_kill(stream);
}

/*
 * The non-blocking form: zero waits for nothing and must return immediately
 * rather than parking. A live query with no traffic is the only reliable way
 * to be sure the queue is empty.
 */
TEST(Stream, NextTimeoutZeroDoesNotBlock) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_stream_t *stream = live_on("quiet_stream");
    if (stream == NULL) {
        TEST_IGNORE_MESSAGE("live queries unavailable on this build");
    }

    sr_notification_t notification;
    int got = sr_stream_next_timeout(stream, &notification, 0);

    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_TIMEOUT, got,
        "an empty queue polled with no timeout should report SR_TIMEOUT");

    sr_stream_kill(stream);
}

/*
 * A timeout must not consume anything. Poll an empty stream until it expires,
 * then produce an event and confirm it is still delivered -- if an expired wait
 * took the notification off the channel, a polling reader would silently lose
 * events, which would be worse than the hang it replaces.
 */
TEST(Stream, ExpiredWaitDoesNotDropNotifications) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_stream_t *stream = live_on("patient_stream");
    if (stream == NULL) {
        TEST_IGNORE_MESSAGE("live queries unavailable on this build");
    }

    sr_notification_t notification;
    for (int i = 0; i < 3; i++) {
        int empty = sr_stream_next_timeout(stream, &notification, 10);
        TEST_ASSERT_EQUAL_INT_MESSAGE(SR_TIMEOUT, empty,
            "nothing has been written, so every poll should expire");
    }

    sr_arr_res_t *res = NULL;
    int n = sr_query(db, &err, &res, "CREATE patient_stream:1 SET v = 1", NULL);
    if (n > 0) sr_arr_res_arr_free(res, n);
    if (n < 0 && err) { sr_string_free(err); err = NULL; }

    int got = sr_stream_next_timeout(stream, &notification, 2000);
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, got,
        "a notification after several expired waits should still arrive");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_ACTION_CREATE, notification.action,
        "and should be the CREATE that produced it");
    sr_notification_free(notification);

    sr_stream_kill(stream);
}

TEST_GROUP_RUNNER(Stream) {
    RUN_TEST_CASE(Stream, Next);
    RUN_TEST_CASE(Stream, NextTimeoutZeroDoesNotBlock);
    RUN_TEST_CASE(Stream, ExpiredWaitDoesNotDropNotifications);
    RUN_TEST_CASE(Stream, Kill);
}
