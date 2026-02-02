#include "surrealdb.h"
#include "unity_fixture.h"
#include <stdio.h>
#include <string.h>

TEST_GROUP(RPC);

TEST_SETUP(RPC) {}

TEST_TEAR_DOWN(RPC) {}

TEST(RPC, New) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};

    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "rpc_new should succeed: %s", err);
        sr_free_string(err);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_OR_EQUAL_INT_MESSAGE(0, result, "rpc_new should succeed");
    TEST_ASSERT_NOT_NULL_MESSAGE(rpc, "RPC handle should not be NULL");

    sr_surreal_rpc_free(rpc);
}

TEST(RPC, Execute) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};

    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }

    // RPC execute requires CBOR-encoded request bytes
    // For now, test with empty/invalid input to verify it handles errors gracefully
    uint8_t *response = NULL;
    uint8_t invalid_cbor[] = {0x00}; // Invalid CBOR

    result = sr_surreal_rpc_execute(rpc, &err, &response, invalid_cbor, 1);
    // This should fail since the CBOR is invalid, but shouldn't crash
    if (result < 0) {
        if (err)
            sr_free_string(err);
    }
    if (response) {
        sr_free_byte_arr(response, result);
    }

    sr_surreal_rpc_free(rpc);
    // Test passes if we get here without crashing
}

TEST(RPC, Notifications) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};

    int result = sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    if (result < 0) {
        if (err)
            sr_free_string(err);
        TEST_FAIL_MESSAGE("Failed to create RPC connection");
    }

    // Get notifications stream
    sr_RpcStream *stream = NULL;
    result = sr_surreal_rpc_notifications(rpc, &err, &stream);
    if (result < 0) {
        if (err)
            sr_free_string(err);
        sr_surreal_rpc_free(rpc);
        TEST_FAIL_MESSAGE("Failed to get notifications stream");
    }
    TEST_ASSERT_NOT_NULL_MESSAGE(stream, "Notifications stream should not be NULL");

    // Clean up
    sr_rpc_stream_free(stream);
    sr_surreal_rpc_free(rpc);
}

TEST(RPC, Free) {
    sr_surreal_rpc_t *rpc;
    sr_string_t err;
    sr_option_t opts = {0};

    sr_surreal_rpc_new(&err, &rpc, "memory", opts);
    sr_surreal_rpc_free(rpc);
    // If we get here without crashing, test passes
}

TEST_GROUP_RUNNER(RPC) {
    RUN_TEST_CASE(RPC, New);
    RUN_TEST_CASE(RPC, Execute);
    RUN_TEST_CASE(RPC, Notifications);
    RUN_TEST_CASE(RPC, Free);
}
