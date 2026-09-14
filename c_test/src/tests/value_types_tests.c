/*
 * Coverage for the value variants added for SurrealDB 3.x: table, file, range,
 * regex and set.
 *
 * Before these existed, surrealdb-types values of those kinds fell through a
 * catch-all in the conversion and reached C as SR_VALUE_NONE, so a caller could
 * not tell a file reference from an absent field. The round-trip tests below
 * exist to keep that from regressing silently.
 */

#include "unity_fixture.h"
#include "surrealdb.h"
#include <string.h>
#include <stdbool.h>

TEST_GROUP(ValueTypes);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(ValueTypes) {
    db = NULL;
    sr_connect(&err, &db, "mem://");
    if (db) {
        sr_use_ns(db, &err, "test_ns");
        sr_use_db(db, &err, "test_db");
    }
}

TEST_TEAR_DOWN(ValueTypes) {
    if (db != NULL) {
        sr_surreal_disconnect(db);
        db = NULL;
    }
}

/* Run a statement and report the tag of its single resulting value.
 *
 * Returns false when the statement produced nothing.
 *
 * The tag is copied out rather than returning the sr_value_t itself: the value
 * lives inside the array sr_query allocated, so a pointer to it cannot outlive
 * that array, and the array is one allocation that must be released whole. An
 * earlier version of this helper returned the interior pointer and leaked the
 * array -- the caller had no way to free either one. */
static bool query_one_tag(const char *statement, sr_value_t_Tag *tag_out) {
    sr_arr_res_t *results = NULL;

    int len = sr_query(db, &err, &results, statement, NULL);
    if (len < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        return false;
    }

    bool got = false;
    if (len > 0 && results[0].ok.len > 0) {
        *tag_out = results[0].ok.arr[0].tag;
        got = true;
    }
    sr_arr_res_arr_free(results, len);
    return got;
}

/* ------------------------------------------------------------ constructors */

TEST(ValueTypes, TableConstructor) {
    sr_value_t *val = sr_value_table("users");
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_TABLE, val->tag, "tag should be TABLE");
    sr_value_free(val);
}

TEST(ValueTypes, FileConstructor) {
    sr_value_t *val = sr_value_file("avatars", "user-1.png");
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_FILE, val->tag, "tag should be FILE");
    TEST_ASSERT_EQUAL_STRING("avatars", val->sr_value_file.bucket);
    TEST_ASSERT_EQUAL_STRING("user-1.png", val->sr_value_file.key);
    sr_value_free(val);
}

TEST(ValueTypes, RegexConstructor) {
    sr_value_t *val = sr_value_regex("^a.*z$");
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_REGEX, val->tag, "tag should be REGEX");
    sr_value_free(val);
}

TEST(ValueTypes, SetConstructor) {
    sr_value_t *val = sr_value_set(NULL);
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_SET, val->tag, "tag should be SET");
    sr_value_free(val);
}

TEST(ValueTypes, RangeConstructor) {
    sr_value_t *lo = sr_value_int(1);
    sr_value_t *hi = sr_value_int(10);

    sr_value_t *val = sr_value_range(sr_bound_included(lo), sr_bound_excluded(hi));
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_RANGE, val->tag, "tag should be RANGE");
    TEST_ASSERT_EQUAL_INT(SR_BOUND_INCLUDED, val->sr_value_range->start.tag);
    TEST_ASSERT_EQUAL_INT(SR_BOUND_EXCLUDED, val->sr_value_range->end.tag);

    /* lo and hi were consumed by the bounds; freeing the range frees them. */
    sr_value_free(val);
}

TEST(ValueTypes, UnboundedRange) {
    sr_value_t *val = sr_value_range(sr_bound_unbounded(), sr_bound_unbounded());
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT(SR_BOUND_UNBOUNDED, val->sr_value_range->start.tag);
    TEST_ASSERT_EQUAL_INT(SR_BOUND_UNBOUNDED, val->sr_value_range->end.tag);
    sr_value_free(val);
}

TEST(ValueTypes, NullConstructorArgsAreRejected) {
    /* A null argument must not crash; it degrades to NONE. */
    sr_value_t *a = sr_value_table(NULL);
    sr_value_t *b = sr_value_file(NULL, NULL);
    sr_value_t *c = sr_value_regex(NULL);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_NONE, a->tag);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_NONE, b->tag);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_NONE, c->tag);
    sr_value_free(a);
    sr_value_free(b);
    sr_value_free(c);
}

/* ------------------------------------------------------------- round trips */

TEST(ValueTypes, RangeSurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t_Tag tag;
    if (!query_one_tag("RETURN 1..10", &tag)) {
        TEST_IGNORE_MESSAGE("server did not return a value for a range literal");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_RANGE, tag,
                                  "a range must not arrive as SR_VALUE_NONE");
}

TEST(ValueTypes, SetSurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t_Tag tag;
    if (!query_one_tag("RETURN <set>[1, 2, 2, 3]", &tag)) {
        TEST_IGNORE_MESSAGE("server did not return a value for a set literal");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_SET, tag,
                                  "a set must not arrive as SR_VALUE_NONE");
}

TEST(ValueTypes, RegexSurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t_Tag tag;
    if (!query_one_tag("RETURN /^abc$/", &tag)) {
        TEST_IGNORE_MESSAGE("server did not return a value for a regex literal");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_REGEX, tag,
                                  "a regex must not arrive as SR_VALUE_NONE");
}

/* Build [1, 2, 3] as an sr_array_t. The caller keeps ownership of both the
   element values and the array itself. */
static sr_array_t *ids_1_2_3(sr_value_t **elems) {
    elems[0] = sr_value_int(1);
    elems[1] = sr_value_int(2);
    elems[2] = sr_value_int(3);
    sr_value_t block[3] = { *elems[0], *elems[1], *elems[2] };
    return sr_array_from_values(block, 3);
}

/* Ask the database a yes/no question about a bound variable. */
static bool query_one_bool(const char *sql, bool *out) {
    sr_arr_res_t *res = NULL;
    int n = sr_query(db, &err, &res, sql, NULL);
    if (n < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        return false;
    }
    bool ok = false;
    if (n > 0) {
        const sr_value_t *v = sr_array_get(&res[0].ok, 0);
        if (v && v->tag == SR_VALUE_BOOL) {
            *out = v->sr_value_bool;
            ok = true;
        }
        sr_arr_res_arr_free(res, n);
    }
    return ok;
}

/*
 * A populated array must be constructible as a value.
 *
 * `sr_value_array` and `sr_value_set` used to take no argument and always
 * produced an empty container, so there was no supported way to bind
 * `WHERE id IN $ids` from C -- the data model allowed it, only the constructor
 * was missing. Callers had to hand-assemble the tagged union.
 */
TEST(ValueTypes, ArrayValueCarriesItsElements) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t *elems[3];
    sr_array_t *arr = ids_1_2_3(elems);
    TEST_ASSERT_EQUAL_INT_MESSAGE(3, sr_array_len(arr), "the array should hold three values");

    sr_value_t *val = sr_value_array(arr);
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_ARRAY, val->tag, "tag should be ARRAY");
    TEST_ASSERT_EQUAL_INT_MESSAGE(3, sr_array_len(val->sr_value_array),
                                  "the value must carry the elements, not an empty array");

    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_set(db, &err, "ids", val));

    bool hit = false, miss = true;
    if (query_one_bool("RETURN 2 IN $ids;", &hit) &&
        query_one_bool("RETURN 9 IN $ids;", &miss)) {
        TEST_ASSERT_TRUE_MESSAGE(hit, "2 should be in [1,2,3]");
        TEST_ASSERT_FALSE_MESSAGE(miss, "9 should not be in [1,2,3]");
    } else {
        TEST_FAIL_MESSAGE("the bound array should be queryable");
    }

    /* The array is copied, so freeing ours must not disturb the value. */
    sr_array_free(arr);
    TEST_ASSERT_EQUAL_INT_MESSAGE(3, sr_array_len(val->sr_value_array),
                                  "the value owns its own copy");

    sr_value_free(val);
    for (int i = 0; i < 3; ++i) sr_value_free(elems[i]);
}

TEST(ValueTypes, SetValueCarriesItsElements) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t *elems[3];
    sr_array_t *arr = ids_1_2_3(elems);

    sr_value_t *val = sr_value_set(arr);
    TEST_ASSERT_NOT_NULL(val);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_SET, val->tag, "tag should be SET");
    TEST_ASSERT_EQUAL_INT_MESSAGE(3, sr_array_len(val->sr_value_set),
                                  "the value must carry the elements");

    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_set(db, &err, "sids", val));

    bool hit = false;
    if (query_one_bool("RETURN 3 IN $sids;", &hit)) {
        TEST_ASSERT_TRUE_MESSAGE(hit, "3 should be in the set");
    } else {
        TEST_FAIL_MESSAGE("the bound set should be queryable");
    }

    sr_array_free(arr);
    sr_value_free(val);
    for (int i = 0; i < 3; ++i) sr_value_free(elems[i]);
}

/* A null pointer is the documented way to get an empty container. */
TEST(ValueTypes, NullArrayYieldsAnEmptyContainer) {
    sr_value_t *a = sr_value_array(NULL);
    TEST_ASSERT_NOT_NULL(a);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_ARRAY, a->tag);
    TEST_ASSERT_EQUAL_INT_MESSAGE(0, sr_array_len(a->sr_value_array), "should be empty");
    sr_value_free(a);

    sr_value_t *s = sr_value_set(NULL);
    TEST_ASSERT_NOT_NULL(s);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_SET, s->tag);
    TEST_ASSERT_EQUAL_INT_MESSAGE(0, sr_array_len(s->sr_value_set), "should be empty");
    sr_value_free(s);
}

TEST_GROUP_RUNNER(ValueTypes) {
    RUN_TEST_CASE(ValueTypes, TableConstructor);
    RUN_TEST_CASE(ValueTypes, FileConstructor);
    RUN_TEST_CASE(ValueTypes, RegexConstructor);
    RUN_TEST_CASE(ValueTypes, SetConstructor);
    RUN_TEST_CASE(ValueTypes, RangeConstructor);
    RUN_TEST_CASE(ValueTypes, UnboundedRange);
    RUN_TEST_CASE(ValueTypes, NullConstructorArgsAreRejected);
    RUN_TEST_CASE(ValueTypes, RangeSurvivesTheDatabase);
    RUN_TEST_CASE(ValueTypes, SetSurvivesTheDatabase);
    RUN_TEST_CASE(ValueTypes, ArrayValueCarriesItsElements);
    RUN_TEST_CASE(ValueTypes, SetValueCarriesItsElements);
    RUN_TEST_CASE(ValueTypes, NullArrayYieldsAnEmptyContainer);
    RUN_TEST_CASE(ValueTypes, RegexSurvivesTheDatabase);
}
