#include <stdio.h>
#include <stdlib.h>
#include <unity.h>
#include "../tests/test_suites.h"

// Unity requires setUp and tearDown functions
void setUp(void) {
    // Called before each test
}

void tearDown(void) {
    // Called after each test
}

int main(int argc, char *argv[]) {
    UNITY_BEGIN();
    
    printf("=============================================================\n");
    printf("SurrealDB C API Test Suite (Unity Framework)\n");
    printf("=============================================================\n\n");
    
    // Connection & Session Management Tests
    RUN_TEST(test_sr_connect);
    RUN_TEST(test_sr_surreal_disconnect);
    RUN_TEST(test_sr_authenticate);
    RUN_TEST(test_sr_invalidate);
    RUN_TEST(test_sr_use_ns);
    RUN_TEST(test_sr_use_db);
    RUN_TEST(test_sr_version);
    RUN_TEST(test_sr_health);
    
    // Transaction Management Tests
    RUN_TEST(test_sr_begin);
    RUN_TEST(test_sr_cancel);
    RUN_TEST(test_sr_commit);
    
    // CRUD Operations Tests
    RUN_TEST(test_sr_create);
    RUN_TEST(test_sr_delete);
    RUN_TEST(test_sr_insert);
    RUN_TEST(test_sr_select);
    RUN_TEST(test_sr_update);
    RUN_TEST(test_sr_upsert);
    RUN_TEST(test_sr_merge);
    
    // Query & Live Select Tests
    RUN_TEST(test_sr_query);
    RUN_TEST(test_sr_select_live);
    
    // Import/Export Tests
    RUN_TEST(test_sr_export);
    RUN_TEST(test_sr_import);
    
    // Authentication Tests
    RUN_TEST(test_sr_signin);
    RUN_TEST(test_sr_signup);
    
    // Variable Management Tests
    RUN_TEST(test_sr_set);
    RUN_TEST(test_sr_unset);
    
    // RPC Tests
    RUN_TEST(test_sr_surreal_rpc_new);
    RUN_TEST(test_sr_surreal_rpc_execute);
    RUN_TEST(test_sr_surreal_rpc_notifications);
    RUN_TEST(test_sr_surreal_rpc_free);
    
    // Stream Tests
    RUN_TEST(test_sr_stream_next);
    RUN_TEST(test_sr_stream_kill);
    
    // Object Manipulation Tests
    RUN_TEST(test_sr_object_insert);
    RUN_TEST(test_sr_object_insert_str);
    RUN_TEST(test_sr_object_insert_int);
    RUN_TEST(test_sr_object_insert_float);
    RUN_TEST(test_sr_object_insert_double);
    
    // Memory Management Tests
    RUN_TEST(test_sr_free_arr);
    RUN_TEST(test_sr_free_bytes);
    RUN_TEST(test_sr_free_byte_arr);
    RUN_TEST(test_sr_free_object);
    RUN_TEST(test_sr_free_arr_res);
    RUN_TEST(test_sr_free_arr_res_arr);
    RUN_TEST(test_sr_free_string);
    
    // Utility Tests
    RUN_TEST(test_sr_print_notification);
    RUN_TEST(test_sr_value_print);
    
    printf("\n=============================================================\n");
    
    return UNITY_END();
}
