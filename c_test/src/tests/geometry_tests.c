/*
 * Geometry values.
 *
 * Previously reachable only through the LegacyApi bridge, which meant the
 * type names went unexercised by anything current -- and those names were
 * wrong: cbindgen's export.prefix stacked onto Rust types already spelled
 * `sr_g_*`, emitting `sr_sr_g_point` and friends. Correcting that collided
 * with the tag enum, whose variants had taken the good names, so the variants
 * are now `SR_GEOMETRY_*` like every other enum in the header and the union
 * members are `sr_geometry_*`.
 *
 * These tests pin the current spelling and exercise every geometry kind
 * through construction, inspection, release and a database round trip.
 */

#include "unity_fixture.h"
#include "surrealdb.h"
#include <stdio.h>
#include <string.h>
#include <math.h>

TEST_GROUP(Geometry);

static sr_surreal_t *db;
static sr_string_t err;

TEST_SETUP(Geometry) {
    db = NULL;
    err = NULL;
    sr_connect(&err, &db, "memory");
    if (db) {
        sr_use_ns(db, &err, "geo_ns");
        sr_use_db(db, &err, "geo_db");
    }
}

TEST_TEAR_DOWN(Geometry) {
    if (err) { sr_string_free(err); err = NULL; }
    if (db) { sr_surreal_disconnect(db); db = NULL; }
}

static sr_g_coord RING[5]  = {{0,0},{10,0},{10,10},{0,10},{0,0}};
static sr_g_coord LINE[3]  = {{0,0},{1,1},{2,0}};

TEST(Geometry, Point) {
    sr_value_t *v = sr_value_point(-122.4194, 37.7749);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_OBJECT, v->tag);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POINT, v->sr_geometry_object.tag);
    /* A point holds an inline coord and owns no array -- which is why it was
       the one geometry kind that never leaked before Drop was added. */
    /* Unity's double assertions are compiled out in this build, so compare
       with an explicit tolerance rather than depend on that configuration. */
    TEST_ASSERT_TRUE_MESSAGE(
        fabs(v->sr_geometry_object.sr_geometry_point._0.x - (-122.4194)) < 1e-9,
        "x should round-trip through the constructor");
    TEST_ASSERT_TRUE_MESSAGE(
        fabs(v->sr_geometry_object.sr_geometry_point._0.y - 37.7749) < 1e-9,
        "y should round-trip through the constructor");
    sr_value_free(v);
}

TEST(Geometry, LineString) {
    sr_value_t *v = sr_value_linestring(LINE, 3);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_LINESTRING, v->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT(3, v->sr_geometry_object.sr_geometry_linestring._0.len);
    sr_value_free(v);
}

TEST(Geometry, Polygon) {
    sr_value_t *v = sr_value_polygon(RING, 5);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POLYGON, v->sr_geometry_object.tag);
    /* Field 0 is the exterior ring; field 1 is the array of interior rings. */
    TEST_ASSERT_EQUAL_INT(5, v->sr_geometry_object.sr_geometry_polygon._0._0.len);
    sr_value_free(v);
}

TEST(Geometry, MultiPoint) {
    sr_value_t *v = sr_value_multipoint(LINE, 3);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_MULTIPOINT, v->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT(3, v->sr_geometry_object.sr_geometry_multipoint._0.len);
    sr_value_free(v);
}

TEST(Geometry, NullArgumentsAreRejected) {
    /* A null coordinate array must degrade, not crash. */
    sr_value_t *a = sr_value_linestring(NULL, 0);
    sr_value_t *b = sr_value_polygon(NULL, 0);
    sr_value_t *c = sr_value_multipoint(NULL, 0);
    if (a) sr_value_free(a);
    if (b) sr_value_free(b);
    if (c) sr_value_free(c);
}

TEST(Geometry, SurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_object_t rec = sr_object_new();
    sr_value_t *area = sr_value_polygon(RING, 5);
    sr_object_insert(&rec, "area", area);
    sr_value_free(area);

    int rc = sr_create(db, &err, NULL, "site:one", &rec);
    sr_object_free(rec);
    if (rc < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("creating a record with a polygon should succeed");
    }

    sr_value_t *rows = NULL;
    int n = sr_select(db, &err, &rows, "site");
    if (n < 0) {
        if (err) { sr_string_free(err); err = NULL; }
        TEST_FAIL_MESSAGE("select should succeed");
    }
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, n, "the record should come back");

    /* The geometry must survive the round trip as a geometry, not collapse to
       SR_VALUE_NONE the way the unmapped value kinds used to. */
    const sr_value_t *area_back = NULL;
    if (rows[0].tag == SR_VALUE_OBJECT) {
        area_back = sr_object_get(&rows[0].sr_value_object, "area");
    }
    TEST_ASSERT_NOT_NULL_MESSAGE(area_back, "the area field should be present");
    if (area_back) {
        TEST_ASSERT_EQUAL_INT_MESSAGE(SR_GEOMETRY_OBJECT, area_back->tag,
                                      "a polygon must not arrive as SR_VALUE_NONE");
        TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POLYGON, area_back->sr_geometry_object.tag);
    }
    sr_values_free(rows, n);
}

TEST(Geometry, RepeatedConstructionDoesNotLeak) {
    /* Exercised under -DSURREALDB_SANITIZE=address: every geometry kind that
       carries an array leaked its payload until Drop was implemented. */
    for (int i = 0; i < 8; ++i) {
        sr_value_free(sr_value_point(1.0, 2.0));
        sr_value_free(sr_value_linestring(LINE, 3));
        sr_value_free(sr_value_polygon(RING, 5));
        sr_value_free(sr_value_multipoint(LINE, 3));
    }
}

/*
 * A GeometryCollection must be constructible in C.
 *
 * It was the one geometry kind with no constructor: the conversion handled it
 * in both directions, but a caller could only obtain one by reading it back
 * from the database. The union member is Rust-allocated, so hand-assembling a
 * collection from a malloc'd block would corrupt the heap on free -- there was
 * no safe workaround, only a clunky one (synthesize via query text, rebind).
 */
TEST(Geometry, Collection) {
    sr_value_t *a = sr_value_point(1.0, 2.0);
    sr_value_t *b = sr_value_linestring(LINE, 3);
    const sr_value_t *members[2] = { a, b };

    sr_value_t *v = sr_value_collection(members, 2);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_OBJECT, v->tag);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_GEOMETRY_COLLECTION, v->sr_geometry_object.tag,
                                  "tag should be COLLECTION");
    TEST_ASSERT_EQUAL_INT_MESSAGE(2, v->sr_geometry_object.sr_geometry_collection.len,
                                  "the collection must carry its members");

    /* The members are copied, so releasing ours must not disturb the value. */
    sr_value_free(a);
    sr_value_free(b);
    TEST_ASSERT_EQUAL_INT_MESSAGE(2, v->sr_geometry_object.sr_geometry_collection.len,
                                  "the value owns its own copy");
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POINT,
                          v->sr_geometry_object.sr_geometry_collection.ptr[0].tag);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_LINESTRING,
                          v->sr_geometry_object.sr_geometry_collection.ptr[1].tag);

    sr_value_free(v);
}

TEST(Geometry, CollectionSurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    sr_value_t *a = sr_value_point(1.0, 2.0);
    sr_value_t *b = sr_value_point(3.0, 4.0);
    const sr_value_t *members[2] = { a, b };
    sr_value_t *v = sr_value_collection(members, 2);

    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_set(db, &err, "g", v));

    /* Into a typed field: the whole point is that a constructed collection
       coerces the same way one read back from the database does. */
    sr_arr_res_t *ddl = NULL;
    int n = sr_query(db, &err, &ddl,
                     "DEFINE TABLE geoc SCHEMAFULL; "
                     "DEFINE FIELD loc ON geoc TYPE geometry<collection>;", NULL);
    if (n > 0) sr_arr_res_arr_free(ddl, n);
    if (err) { sr_string_free(err); err = NULL; }

    sr_arr_res_t *res = NULL;
    int rc = sr_query(db, &err, &res, "CREATE geoc:1 SET loc = $g RETURN loc;", NULL);
    if (rc < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "a constructed collection should store: %s",
                 err ? (const char *)err : "(no error)");
        if (err) { sr_string_free(err); err = NULL; }
        sr_value_free(v); sr_value_free(a); sr_value_free(b);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_THAN_INT_MESSAGE(0, rc, "the create should return a row");

    const sr_value_t *row = sr_array_get(&res[0].ok, 0);
    const sr_value_t *back = NULL;
    if (row && row->tag == SR_VALUE_OBJECT) {
        back = sr_object_get(&row->sr_value_object, "loc");
    }
    TEST_ASSERT_NOT_NULL_MESSAGE(back, "the loc field should be present");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_GEOMETRY_OBJECT, back->tag,
                                  "a collection must not arrive as SR_VALUE_NONE");
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_GEOMETRY_COLLECTION, back->sr_geometry_object.tag,
                                  "it must still be a collection");

    sr_arr_res_arr_free(res, rc);
    sr_value_free(v);
    sr_value_free(a);
    sr_value_free(b);
}

TEST(Geometry, CollectionRejectsNonGeometryMembers) {
    /* A null array is an empty collection, like sr_value_array(NULL). */
    sr_value_t *empty = sr_value_collection(NULL, 0);
    TEST_ASSERT_NOT_NULL(empty);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_OBJECT, empty->tag);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_COLLECTION, empty->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT(0, empty->sr_geometry_object.sr_geometry_collection.len);
    sr_value_free(empty);

    /* A non-geometry member degrades to NONE rather than a malformed value. */
    sr_value_t *pt = sr_value_point(1.0, 2.0);
    sr_value_t *num = sr_value_int(7);
    const sr_value_t *mixed[2] = { pt, num };
    sr_value_t *bad = sr_value_collection(mixed, 2);
    TEST_ASSERT_NOT_NULL(bad);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NONE, bad->tag,
                                  "a non-geometry member must be rejected");
    sr_value_free(bad);

    /* So does a null member. */
    const sr_value_t *holed[2] = { pt, NULL };
    sr_value_t *holed_v = sr_value_collection(holed, 2);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_NONE, holed_v->tag);
    sr_value_free(holed_v);

    sr_value_free(pt);
    sr_value_free(num);
}

/* An exterior ring and a hole inside it. */
static sr_g_coord HOLE[5] = {{2,2},{4,2},{4,4},{2,4},{2,2}};

/*
 * A polygon with a hole must be constructible in C.
 *
 * sr_value_polygon takes one flat ring and hardcodes an empty interior list,
 * so nothing built in C could have a hole -- and the failure was silent: the
 * constructor returned a perfectly valid hole-less polygon with no signal that
 * anything was missing. Reading a holed polygon back always worked, so data
 * round-tripped through C was quietly flattened.
 */
TEST(Geometry, PolygonWithHole) {
    const sr_g_coord *rings[2] = { RING, HOLE };
    int lens[2] = { 5, 5 };

    sr_value_t *v = sr_value_polygon_rings(rings, lens, 2);
    TEST_ASSERT_NOT_NULL(v);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_OBJECT, v->tag);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POLYGON, v->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT_MESSAGE(5, v->sr_geometry_object.sr_geometry_polygon._0._0.len,
                                  "the exterior ring should carry five coords");
    TEST_ASSERT_EQUAL_INT_MESSAGE(1, v->sr_geometry_object.sr_geometry_polygon._1.len,
                                  "the hole must be present");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        5, v->sr_geometry_object.sr_geometry_polygon._1.ptr[0]._0.len,
        "the hole should carry five coords");
    sr_value_free(v);

    /* One ring is just a polygon with no holes -- same as sr_value_polygon. */
    sr_value_t *solid = sr_value_polygon_rings(rings, lens, 1);
    TEST_ASSERT_EQUAL_INT(0, solid->sr_geometry_object.sr_geometry_polygon._1.len);
    sr_value_free(solid);

    /* Null degrades to an empty polygon rather than crashing. */
    sr_value_t *empty = sr_value_polygon_rings(NULL, NULL, 0);
    TEST_ASSERT_NOT_NULL(empty);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POLYGON, empty->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT(0, empty->sr_geometry_object.sr_geometry_polygon._1.len);
    sr_value_free(empty);
}

TEST(Geometry, PolygonHoleSurvivesTheDatabase) {
    TEST_ASSERT_NOT_NULL_MESSAGE(db, "Connection should succeed");

    const sr_g_coord *rings[2] = { RING, HOLE };
    int lens[2] = { 5, 5 };
    sr_value_t *v = sr_value_polygon_rings(rings, lens, 2);

    TEST_ASSERT_GREATER_OR_EQUAL_INT(0, sr_set(db, &err, "p", v));

    sr_arr_res_t *res = NULL;
    int rc = sr_query(db, &err, &res, "RETURN $p;", NULL);
    if (rc < 0) {
        char msg[256];
        snprintf(msg, sizeof(msg), "a holed polygon should bind: %s",
                 err ? (const char *)err : "(no error)");
        if (err) { sr_string_free(err); err = NULL; }
        sr_value_free(v);
        TEST_FAIL_MESSAGE(msg);
    }
    TEST_ASSERT_GREATER_THAN_INT(0, rc);

    const sr_value_t *back = sr_array_get(&res[0].ok, 0);
    TEST_ASSERT_NOT_NULL(back);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_GEOMETRY_OBJECT, back->tag,
                                  "a polygon must not arrive as SR_VALUE_NONE");
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_POLYGON, back->sr_geometry_object.tag);
    /* The whole point: the hole is still there after the round trip. */
    TEST_ASSERT_EQUAL_INT_MESSAGE(1, back->sr_geometry_object.sr_geometry_polygon._1.len,
                                  "the hole must survive the database");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        5, back->sr_geometry_object.sr_geometry_polygon._1.ptr[0]._0.len,
        "the hole should still carry five coords");

    sr_arr_res_arr_free(res, rc);
    sr_value_free(v);
}

TEST(Geometry, MultiPolygonFromPolygonValues) {
    const sr_g_coord *rings[2] = { RING, HOLE };
    int lens[2] = { 5, 5 };

    sr_value_t *holed = sr_value_polygon_rings(rings, lens, 2);
    sr_value_t *solid = sr_value_polygon(RING, 5);
    const sr_value_t *members[2] = { holed, solid };

    sr_value_t *mp = sr_value_multipolygon_from(members, 2);
    TEST_ASSERT_NOT_NULL(mp);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_MULTIPOLYGON, mp->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT_MESSAGE(2, mp->sr_geometry_object.sr_geometry_multipolygon._0.len,
                                  "both polygons should be carried");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        1, mp->sr_geometry_object.sr_geometry_multipolygon._0.ptr[0]._1.len,
        "the first member must keep its hole");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        0, mp->sr_geometry_object.sr_geometry_multipolygon._0.ptr[1]._1.len,
        "the second member has none");

    /* Members are copied, so releasing ours must not disturb the value. */
    sr_value_free(holed);
    sr_value_free(solid);
    TEST_ASSERT_EQUAL_INT(2, mp->sr_geometry_object.sr_geometry_multipolygon._0.len);
    TEST_ASSERT_EQUAL_INT(1, mp->sr_geometry_object.sr_geometry_multipolygon._0.ptr[0]._1.len);
    sr_value_free(mp);
}

TEST(Geometry, MultiPolygonFromRejectsNonPolygons) {
    sr_value_t *empty = sr_value_multipolygon_from(NULL, 0);
    TEST_ASSERT_NOT_NULL(empty);
    TEST_ASSERT_EQUAL_INT(SR_GEOMETRY_MULTIPOLYGON, empty->sr_geometry_object.tag);
    TEST_ASSERT_EQUAL_INT(0, empty->sr_geometry_object.sr_geometry_multipolygon._0.len);
    sr_value_free(empty);

    /* A point is a geometry but not a polygon -- still rejected. */
    sr_value_t *poly = sr_value_polygon(RING, 5);
    sr_value_t *pt = sr_value_point(1.0, 2.0);
    const sr_value_t *mixed[2] = { poly, pt };
    sr_value_t *bad = sr_value_multipolygon_from(mixed, 2);
    TEST_ASSERT_EQUAL_INT_MESSAGE(SR_VALUE_NONE, bad->tag,
                                  "a non-polygon member must be rejected");
    sr_value_free(bad);

    const sr_value_t *holed[2] = { poly, NULL };
    sr_value_t *holed_v = sr_value_multipolygon_from(holed, 2);
    TEST_ASSERT_EQUAL_INT(SR_VALUE_NONE, holed_v->tag);
    sr_value_free(holed_v);

    sr_value_free(poly);
    sr_value_free(pt);
}

TEST_GROUP_RUNNER(Geometry) {
    RUN_TEST_CASE(Geometry, Point);
    RUN_TEST_CASE(Geometry, LineString);
    RUN_TEST_CASE(Geometry, Polygon);
    RUN_TEST_CASE(Geometry, MultiPoint);
    RUN_TEST_CASE(Geometry, NullArgumentsAreRejected);
    RUN_TEST_CASE(Geometry, SurvivesTheDatabase);
    RUN_TEST_CASE(Geometry, RepeatedConstructionDoesNotLeak);
    RUN_TEST_CASE(Geometry, Collection);
    RUN_TEST_CASE(Geometry, CollectionSurvivesTheDatabase);
    RUN_TEST_CASE(Geometry, CollectionRejectsNonGeometryMembers);
    RUN_TEST_CASE(Geometry, PolygonWithHole);
    RUN_TEST_CASE(Geometry, PolygonHoleSurvivesTheDatabase);
    RUN_TEST_CASE(Geometry, MultiPolygonFromPolygonValues);
    RUN_TEST_CASE(Geometry, MultiPolygonFromRejectsNonPolygons);
}
