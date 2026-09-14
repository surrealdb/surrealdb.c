/*
 * Unity bridge for the api_tests corpus.
 *
 * api_tests.c holds self-contained test bodies that predate the Unity suite and
 * still carry coverage it does not: the geometry and value constructors, array
 * and patch operations, sr_relate and sr_kill among others. Rather than rewrite
 * them and risk changing what they assert, they are driven from Unity here, so
 * that a single corpus sits behind all three drivers.
 *
 * One group, not one per cluster: Unity Fixture matches "-g" with strstr(), so
 * a group called "ApiUtility" would also be selected by "-g Utility".
 *
 * On failure the underlying ASSERT_* macros print the failing expression with
 * its file and line to stderr; Unity reports which test it was.
 *
 * Generated so that every function in api_tests.c is reached. Regenerate rather
 * than hand-edit when adding to api_tests.c.
 */

#include "unity_fixture.h"

#include "api_tests.h"

TEST_GROUP(LegacyApi);

TEST_SETUP(LegacyApi) {}

TEST_TEAR_DOWN(LegacyApi) {}

/* These bodies report a deliberate skip as API_TEST_SKIP rather than failing. */
#define BRIDGE_TEST(name, fn)                                                  \
    TEST(LegacyApi, name)                                                        \
    {                                                                          \
        int rc = fn();                                                         \
        if (rc == API_TEST_SKIP)                                               \
        {                                                                      \
            TEST_IGNORE_MESSAGE("skipped by the test body");                   \
        }                                                                      \
        TEST_ASSERT_EQUAL_INT_MESSAGE(API_TEST_PASS, rc,                       \
                                      "failing assertion printed to stderr");  \
    }

BRIDGE_TEST(array_get, test_sr_array_get)
BRIDGE_TEST(array_len, test_sr_array_len)
BRIDGE_TEST(array_push, test_sr_array_push)
BRIDGE_TEST(authenticate, test_sr_authenticate)
BRIDGE_TEST(begin, test_sr_begin)
BRIDGE_TEST(cancel, test_sr_cancel)
BRIDGE_TEST(commit, test_sr_commit)
BRIDGE_TEST(connect, test_sr_connect)
BRIDGE_TEST(create, test_sr_create)
BRIDGE_TEST(delete, test_sr_delete)
BRIDGE_TEST(export, test_sr_export)
BRIDGE_TEST(health, test_sr_health)
BRIDGE_TEST(import, test_sr_import)
BRIDGE_TEST(insert, test_sr_insert)
BRIDGE_TEST(insert_relation, test_sr_insert_relation)
BRIDGE_TEST(invalidate, test_sr_invalidate)
BRIDGE_TEST(kill, test_sr_kill)
BRIDGE_TEST(merge, test_sr_merge)
BRIDGE_TEST(object_free, test_sr_object_free)
BRIDGE_TEST(object_get, test_sr_object_get)
BRIDGE_TEST(object_insert, test_sr_object_insert)
BRIDGE_TEST(object_insert_double, test_sr_object_insert_double)
BRIDGE_TEST(object_insert_float, test_sr_object_insert_float)
BRIDGE_TEST(object_insert_int, test_sr_object_insert_int)
BRIDGE_TEST(object_insert_str, test_sr_object_insert_str)
BRIDGE_TEST(object_keys, test_sr_object_keys)
BRIDGE_TEST(object_len, test_sr_object_len)
BRIDGE_TEST(object_new, test_sr_object_new)
BRIDGE_TEST(patch_add, test_sr_patch_add)
BRIDGE_TEST(patch_remove, test_sr_patch_remove)
BRIDGE_TEST(patch_replace, test_sr_patch_replace)
BRIDGE_TEST(print_notification, test_sr_print_notification)
BRIDGE_TEST(query, test_sr_query)
BRIDGE_TEST(relate, test_sr_relate)
BRIDGE_TEST(rpc_stream_free, test_sr_rpc_stream_free)
BRIDGE_TEST(rpc_stream_next, test_sr_rpc_stream_next)
BRIDGE_TEST(run, test_sr_run)
BRIDGE_TEST(select, test_sr_select)
BRIDGE_TEST(select_live, test_sr_select_live)
BRIDGE_TEST(set, test_sr_set)
BRIDGE_TEST(signin, test_sr_signin)
BRIDGE_TEST(signup, test_sr_signup)
BRIDGE_TEST(stream_kill, test_sr_stream_kill)
BRIDGE_TEST(stream_next, test_sr_stream_next)
BRIDGE_TEST(string_free, test_sr_string_free)
BRIDGE_TEST(surreal_disconnect, test_sr_surreal_disconnect)
BRIDGE_TEST(surreal_rpc_execute, test_sr_surreal_rpc_execute)
BRIDGE_TEST(surreal_rpc_free, test_sr_surreal_rpc_free)
BRIDGE_TEST(surreal_rpc_new, test_sr_surreal_rpc_new)
BRIDGE_TEST(surreal_rpc_notifications, test_sr_surreal_rpc_notifications)
BRIDGE_TEST(unset, test_sr_unset)
BRIDGE_TEST(update, test_sr_update)
BRIDGE_TEST(upsert, test_sr_upsert)
BRIDGE_TEST(use_db, test_sr_use_db)
BRIDGE_TEST(use_ns, test_sr_use_ns)
BRIDGE_TEST(value_array, test_sr_value_array)
BRIDGE_TEST(value_bool, test_sr_value_bool)
BRIDGE_TEST(value_bytes, test_sr_value_bytes)
BRIDGE_TEST(value_datetime, test_sr_value_datetime)
BRIDGE_TEST(value_decimal, test_sr_value_decimal)
BRIDGE_TEST(value_duration, test_sr_value_duration)
BRIDGE_TEST(value_eq, test_sr_value_eq)
BRIDGE_TEST(value_float, test_sr_value_float)
BRIDGE_TEST(value_free, test_sr_value_free)
BRIDGE_TEST(value_int, test_sr_value_int)
BRIDGE_TEST(value_linestring, test_sr_value_linestring)
BRIDGE_TEST(value_multilinestring, test_sr_value_multilinestring)
BRIDGE_TEST(value_multipoint, test_sr_value_multipoint)
BRIDGE_TEST(value_multipolygon, test_sr_value_multipolygon)
BRIDGE_TEST(value_none, test_sr_value_none)
BRIDGE_TEST(value_null, test_sr_value_null)
BRIDGE_TEST(value_object, test_sr_value_object)
BRIDGE_TEST(value_point, test_sr_value_point)
BRIDGE_TEST(value_polygon, test_sr_value_polygon)
BRIDGE_TEST(value_print, test_sr_value_print)
BRIDGE_TEST(value_string, test_sr_value_string)
BRIDGE_TEST(value_thing, test_sr_value_thing)
BRIDGE_TEST(value_uuid, test_sr_value_uuid)
BRIDGE_TEST(values_free, test_sr_values_free)
BRIDGE_TEST(version, test_sr_version)

TEST_GROUP_RUNNER(LegacyApi)
{
    RUN_TEST_CASE(LegacyApi, array_get);
    RUN_TEST_CASE(LegacyApi, array_len);
    RUN_TEST_CASE(LegacyApi, array_push);
    RUN_TEST_CASE(LegacyApi, authenticate);
    RUN_TEST_CASE(LegacyApi, begin);
    RUN_TEST_CASE(LegacyApi, cancel);
    RUN_TEST_CASE(LegacyApi, commit);
    RUN_TEST_CASE(LegacyApi, connect);
    RUN_TEST_CASE(LegacyApi, create);
    RUN_TEST_CASE(LegacyApi, delete);
    RUN_TEST_CASE(LegacyApi, export);
    RUN_TEST_CASE(LegacyApi, health);
    RUN_TEST_CASE(LegacyApi, import);
    RUN_TEST_CASE(LegacyApi, insert);
    RUN_TEST_CASE(LegacyApi, insert_relation);
    RUN_TEST_CASE(LegacyApi, invalidate);
    RUN_TEST_CASE(LegacyApi, kill);
    RUN_TEST_CASE(LegacyApi, merge);
    RUN_TEST_CASE(LegacyApi, object_free);
    RUN_TEST_CASE(LegacyApi, object_get);
    RUN_TEST_CASE(LegacyApi, object_insert);
    RUN_TEST_CASE(LegacyApi, object_insert_double);
    RUN_TEST_CASE(LegacyApi, object_insert_float);
    RUN_TEST_CASE(LegacyApi, object_insert_int);
    RUN_TEST_CASE(LegacyApi, object_insert_str);
    RUN_TEST_CASE(LegacyApi, object_keys);
    RUN_TEST_CASE(LegacyApi, object_len);
    RUN_TEST_CASE(LegacyApi, object_new);
    RUN_TEST_CASE(LegacyApi, patch_add);
    RUN_TEST_CASE(LegacyApi, patch_remove);
    RUN_TEST_CASE(LegacyApi, patch_replace);
    RUN_TEST_CASE(LegacyApi, print_notification);
    RUN_TEST_CASE(LegacyApi, query);
    RUN_TEST_CASE(LegacyApi, relate);
    RUN_TEST_CASE(LegacyApi, rpc_stream_free);
    RUN_TEST_CASE(LegacyApi, rpc_stream_next);
    RUN_TEST_CASE(LegacyApi, run);
    RUN_TEST_CASE(LegacyApi, select);
    RUN_TEST_CASE(LegacyApi, select_live);
    RUN_TEST_CASE(LegacyApi, set);
    RUN_TEST_CASE(LegacyApi, signin);
    RUN_TEST_CASE(LegacyApi, signup);
    RUN_TEST_CASE(LegacyApi, stream_kill);
    RUN_TEST_CASE(LegacyApi, stream_next);
    RUN_TEST_CASE(LegacyApi, string_free);
    RUN_TEST_CASE(LegacyApi, surreal_disconnect);
    RUN_TEST_CASE(LegacyApi, surreal_rpc_execute);
    RUN_TEST_CASE(LegacyApi, surreal_rpc_free);
    RUN_TEST_CASE(LegacyApi, surreal_rpc_new);
    RUN_TEST_CASE(LegacyApi, surreal_rpc_notifications);
    RUN_TEST_CASE(LegacyApi, unset);
    RUN_TEST_CASE(LegacyApi, update);
    RUN_TEST_CASE(LegacyApi, upsert);
    RUN_TEST_CASE(LegacyApi, use_db);
    RUN_TEST_CASE(LegacyApi, use_ns);
    RUN_TEST_CASE(LegacyApi, value_array);
    RUN_TEST_CASE(LegacyApi, value_bool);
    RUN_TEST_CASE(LegacyApi, value_bytes);
    RUN_TEST_CASE(LegacyApi, value_datetime);
    RUN_TEST_CASE(LegacyApi, value_decimal);
    RUN_TEST_CASE(LegacyApi, value_duration);
    RUN_TEST_CASE(LegacyApi, value_eq);
    RUN_TEST_CASE(LegacyApi, value_float);
    RUN_TEST_CASE(LegacyApi, value_free);
    RUN_TEST_CASE(LegacyApi, value_int);
    RUN_TEST_CASE(LegacyApi, value_linestring);
    RUN_TEST_CASE(LegacyApi, value_multilinestring);
    RUN_TEST_CASE(LegacyApi, value_multipoint);
    RUN_TEST_CASE(LegacyApi, value_multipolygon);
    RUN_TEST_CASE(LegacyApi, value_none);
    RUN_TEST_CASE(LegacyApi, value_null);
    RUN_TEST_CASE(LegacyApi, value_object);
    RUN_TEST_CASE(LegacyApi, value_point);
    RUN_TEST_CASE(LegacyApi, value_polygon);
    RUN_TEST_CASE(LegacyApi, value_print);
    RUN_TEST_CASE(LegacyApi, value_string);
    RUN_TEST_CASE(LegacyApi, value_thing);
    RUN_TEST_CASE(LegacyApi, value_uuid);
    RUN_TEST_CASE(LegacyApi, values_free);
    RUN_TEST_CASE(LegacyApi, version);
}
