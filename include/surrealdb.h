#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define sr_SR_NONE 0

#define sr_SR_CLOSED -1

#define sr_SR_ERROR -2

#define sr_SR_FATAL -3

typedef enum sr_credentials_scope {
  ROOT,
  NAMESPACE,
  DATABASE,
  RECORD,
} sr_credentials_scope;

typedef enum sr_action {
  SR_ACTION_CREATE,
  SR_ACTION_UPDATE,
  SR_ACTION_DELETE,
  /**
   * Represents an action type added in a newer version of SurrealDB
   * that this C API version doesn't yet support
   */
  SR_ACTION_UNIMPLEMENTED,
} sr_action;

typedef struct sr_opaque_object_internal_t sr_opaque_object_internal_t;

typedef struct sr_RpcStream sr_RpcStream;

/**
 * may be sent across threads, but must not be aliased
 */
typedef struct sr_stream_t sr_stream_t;

/**
 * The object representing a Surreal connection
 *
 * It is safe to be referenced from multiple threads
 * If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
 * (use will cause the program to abort)
 *
 * should be freed with sr_surreal_disconnect
 */
typedef struct sr_surreal_t sr_surreal_t;

/**
 * The object representing a Surreal connection
 *
 * It is safe to be referenced from multiple threads
 * If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
 * (use will cause the program to abort)
 *
 * should be freed with sr_surreal_disconnect
 */
typedef struct sr_surreal_rpc_t sr_surreal_rpc_t;

typedef char *sr_string_t;

typedef struct sr_object_t {
  struct sr_opaque_object_internal_t *_0;
} sr_object_t;

typedef enum sr_number_t_Tag {
  SR_NUMBER_INT,
  SR_NUMBER_FLOAT,
  /**
   * Decimal stored as string representation for C compatibility
   */
  SR_NUMBER_DECIMAL,
} sr_number_t_Tag;

typedef struct sr_number_t {
  sr_number_t_Tag tag;
  union {
    struct {
      int64_t sr_number_int;
    };
    struct {
      double sr_number_float;
    };
    struct {
      sr_string_t sr_number_decimal;
    };
  };
} sr_number_t;

typedef struct sr_duration_t {
  uint64_t secs;
  uint32_t nanos;
} sr_duration_t;

typedef struct sr_uuid_t {
  uint8_t _0[16];
} sr_uuid_t;

typedef struct sr_array_t {
  struct sr_value_t *arr;
  int len;
} sr_array_t;

typedef struct sr_sr_g_coord {
  double x;
  double y;
} sr_sr_g_coord;

typedef struct sr_sr_g_point {
  struct sr_sr_g_coord _0;
} sr_sr_g_point;

typedef struct sr_ArrayGen_sr_g_coord {
  struct sr_sr_g_coord *ptr;
  int len;
} sr_ArrayGen_sr_g_coord;

typedef struct sr_sr_g_linestring {
  struct sr_ArrayGen_sr_g_coord _0;
} sr_sr_g_linestring;

typedef struct sr_ArrayGen_sr_g_linestring {
  struct sr_sr_g_linestring *ptr;
  int len;
} sr_ArrayGen_sr_g_linestring;

typedef struct sr_sr_g_polygon {
  struct sr_sr_g_linestring _0;
  struct sr_ArrayGen_sr_g_linestring _1;
} sr_sr_g_polygon;

typedef struct sr_ArrayGen_sr_g_point {
  struct sr_sr_g_point *ptr;
  int len;
} sr_ArrayGen_sr_g_point;

typedef struct sr_sr_g_multipoint {
  struct sr_ArrayGen_sr_g_point _0;
} sr_sr_g_multipoint;

typedef struct sr_sr_g_multilinestring {
  struct sr_ArrayGen_sr_g_linestring _0;
} sr_sr_g_multilinestring;

typedef struct sr_ArrayGen_sr_g_polygon {
  struct sr_sr_g_polygon *ptr;
  int len;
} sr_ArrayGen_sr_g_polygon;

typedef struct sr_sr_g_multipolygon {
  struct sr_ArrayGen_sr_g_polygon _0;
} sr_sr_g_multipolygon;

typedef struct sr_ArrayGen_sr_geometry {
  struct sr_sr_geometry *ptr;
  int len;
} sr_ArrayGen_sr_geometry;

typedef enum sr_sr_geometry_Tag {
  sr_g_point,
  sr_g_linestring,
  sr_g_polygon,
  sr_g_multipoint,
  sr_g_multiline,
  sr_g_multipolygon,
  sr_g_collection,
  /**
   * Represents a geometry type added in a newer version of SurrealDB
   * that this C API version doesn't yet support
   */
  sr_g_unimplemented,
} sr_sr_geometry_Tag;

typedef struct sr_sr_geometry {
  sr_sr_geometry_Tag tag;
  union {
    struct {
      struct sr_sr_g_point sr_g_point;
    };
    struct {
      struct sr_sr_g_linestring sr_g_linestring;
    };
    struct {
      struct sr_sr_g_polygon sr_g_polygon;
    };
    struct {
      struct sr_sr_g_multipoint sr_g_multipoint;
    };
    struct {
      struct sr_sr_g_multilinestring sr_g_multiline;
    };
    struct {
      struct sr_sr_g_multipolygon sr_g_multipolygon;
    };
    struct {
      struct sr_ArrayGen_sr_geometry sr_g_collection;
    };
  };
} sr_sr_geometry;

typedef struct sr_bytes_t {
  uint8_t *arr;
  int len;
} sr_bytes_t;

typedef enum sr_id_t_Tag {
  SR_ID_NUMBER,
  SR_ID_STRING,
  SR_ID_ARRAY,
  SR_ID_OBJECT,
} sr_id_t_Tag;

typedef struct sr_id_t {
  sr_id_t_Tag tag;
  union {
    struct {
      int64_t sr_id_number;
    };
    struct {
      sr_string_t sr_id_string;
    };
    struct {
      struct sr_array_t *sr_id_array;
    };
    struct {
      struct sr_object_t sr_id_object;
    };
  };
} sr_id_t;

typedef struct sr_thing_t {
  sr_string_t table;
  struct sr_id_t id;
} sr_thing_t;

typedef enum sr_value_t_Tag {
  SR_VALUE_NONE,
  SR_VALUE_NULL,
  SR_VALUE_BOOL,
  SR_VALUE_NUMBER,
  SR_VALUE_STRAND,
  SR_VALUE_DURATION,
  SR_VALUE_DATETIME,
  SR_VALUE_UUID,
  SR_VALUE_ARRAY,
  SR_VALUE_OBJECT,
  SR_GEOMETRY_OBJECT,
  SR_VALUE_BYTES,
  SR_VALUE_THING,
} sr_value_t_Tag;

typedef struct sr_value_t {
  sr_value_t_Tag tag;
  union {
    struct {
      bool sr_value_bool;
    };
    struct {
      struct sr_number_t sr_value_number;
    };
    struct {
      sr_string_t sr_value_strand;
    };
    struct {
      struct sr_duration_t sr_value_duration;
    };
    struct {
      sr_string_t sr_value_datetime;
    };
    struct {
      struct sr_uuid_t sr_value_uuid;
    };
    struct {
      struct sr_array_t *sr_value_array;
    };
    struct {
      struct sr_object_t sr_value_object;
    };
    struct {
      struct sr_sr_geometry sr_geometry_object;
    };
    struct {
      struct sr_bytes_t sr_value_bytes;
    };
    struct {
      struct sr_thing_t sr_value_thing;
    };
  };
} sr_value_t;

/**
 * when code = 0 there is no error
 */
typedef struct sr_SurrealError {
  int code;
  sr_string_t msg;
} sr_SurrealError;

typedef struct sr_arr_res_t {
  struct sr_array_t ok;
  struct sr_SurrealError err;
} sr_arr_res_t;

typedef struct sr_credentials {
  sr_string_t username;
  sr_string_t password;
} sr_credentials;

typedef struct sr_credentials_access {
  sr_string_t namespace_;
  sr_string_t database;
  sr_string_t access;
} sr_credentials_access;

typedef struct sr_option_t {
  bool strict;
  uint8_t query_timeout;
  uint8_t transaction_timeout;
} sr_option_t;

typedef struct sr_notification_t {
  struct sr_uuid_t query_id;
  enum sr_action action;
  struct sr_value_t data;
} sr_notification_t;

/**
 * connects to a local, remote, or embedded database
 *
 * if any function returns SR_FATAL, this must not be used (except to drop) (TODO: check this is safe) doing so will cause the program to abort
 *
 * # Examples
 *
 * ```c
 * sr_string_t err;
 * sr_surreal_t *db;
 *
 * // connect to in-memory instance
 * if (sr_connect(&err, &db, "mem://") < 0) {
 *     printf("error connecting to db: %s\n", err);
 *     return 1;
 * }
 *
 * // connect to surrealkv file
 * if (sr_connect(&err, &db, "surrealkv://test.skv") < 0) {
 *     printf("error connecting to db: %s\n", err);
 *     return 1;
 * }
 *
 * // connect to surrealdb server
 * if (sr_connect(&err, &db, "wss://localhost:8000") < 0) {
 *     printf("error connecting to db: %s\n", err);
 *     return 1;
 * }
 *
 * sr_surreal_disconnect(db);
 * ```
 */
int sr_connect(sr_string_t *err_ptr,
               struct sr_surreal_t **surreal_ptr,
               const char *endpoint);

/**
 * disconnect a database connection
 * note: the Surreal object must not be used after this function has been called
 *     any object allocations will still be valid, and should be freed, using the appropriate function
 * TODO: check if Stream can be freed after disconnection because of rt
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * // connect
 * disconnect(db);
 * ```
 */
void sr_surreal_disconnect(struct sr_surreal_t *db);

/**
 * authenticate with a token
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * const sr_string_t token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...";
 * if (sr_authenticate(db, &err, token) < 0) {
 *     printf("Failed to authenticate: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_authenticate(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *token);

/**
 * begin a new transaction
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_begin(db, &err) < 0) {
 *     printf("Failed to begin transaction: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_begin(const struct sr_surreal_t *db, sr_string_t *err_ptr);

/**
 * cancel the current transaction
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_cancel(db, &err) < 0) {
 *     printf("Failed to cancel transaction: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_cancel(const struct sr_surreal_t *db, sr_string_t *err_ptr);

/**
 * commit the current transaction
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_commit(db, &err) < 0) {
 *     printf("Failed to commit transaction: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_commit(const struct sr_surreal_t *db, sr_string_t *err_ptr);

/**
 * create a record
 *
 */
int sr_create(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_object_t **res_ptr,
              const char *resource,
              const struct sr_object_t *content);

/**
 * delete a record or records
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *deleted;
 * int len = sr_delete(db, &err, &deleted, "foo:bar");
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * sr_free_arr(deleted, len);
 * ```
 */
int sr_delete(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource);

/**
 * export database data to a file
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_export(db, &err, "backup.surql") < 0) {
 *     printf("Export failed: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_export(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *file_path);

/**
 * check the health of the database server
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_health(db, &err) < 0) {
 *     printf("Database unhealthy: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_health(const struct sr_surreal_t *db, sr_string_t *err_ptr);

/**
 * import database data from a file
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_import(db, &err, "backup.surql") < 0) {
 *     printf("Import failed: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_import(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *file_path);

/**
 * insert one or more records
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *inserted;
 * sr_object_t *content = ...; // create content object
 * int len = sr_insert(db, &err, &inserted, "foo", content);
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * sr_free_arr(inserted, len);
 * ```
 */
int sr_insert(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource,
              const struct sr_object_t *content);

/**
 * Insert a relation between records
 *
 * The content object must contain 'in' and 'out' fields specifying the records to relate.
 * Additional fields can be added as relation properties.
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *result;
 * sr_object_t *content = sr_object_new();
 * sr_object_insert_str(content, "in", "person:john");
 * sr_object_insert_str(content, "out", "person:jane");
 * sr_object_insert_str(content, "met", "2024-01-01");
 * int len = sr_insert_relation(db, &err, &result, "knows", content);
 * if (len < 0) {
 *     printf("Failed to insert relation: %s", err);
 *     return 1;
 * }
 * sr_free_arr(result, len);
 * ```
 */
int sr_insert_relation(const struct sr_surreal_t *db,
                       sr_string_t *err_ptr,
                       struct sr_value_t **res_ptr,
                       const char *table,
                       const struct sr_object_t *content);

/**
 * Execute a SurrealDB function
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *result;
 * sr_array_t args = ...; // create args array
 * if (sr_run(db, &err, &result, "fn::my_function", &args) < 0) {
 *     printf("Failed to run function: %s", err);
 *     return 1;
 * }
 * sr_free_arr(result, 1);
 * ```
 */
int sr_run(const struct sr_surreal_t *db,
           sr_string_t *err_ptr,
           struct sr_value_t **res_ptr,
           const char *function_name,
           const struct sr_array_t *args);

/**
 * Create a graph relation between two records
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *result;
 * sr_object_t *content = sr_object_new();
 * int len = sr_relate(db, &err, &result, "person:john", "knows", "person:jane", content);
 * if (len < 0) {
 *     printf("Failed to create relation: %s", err);
 *     return 1;
 * }
 * sr_free_arr(result, len);
 * ```
 */
int sr_relate(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *from,
              const char *relation,
              const char *to,
              const struct sr_object_t *content);

/**
 * invalidate the current authentication session
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_invalidate(db, &err) < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_invalidate(const struct sr_surreal_t *db, sr_string_t *err_ptr);

/**
 * make a live selection
 * if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
 * which can be moved across threads but not aliased
 *
 * # Examples
 *
 * sr_stream_t *stream;
 * if (sr_select_live(db, &err, &stream, "foo") < 0)
 * {
 *     printf("%s", err);
 *     return 1;
 * }
 *
 * sr_notification_t not ;
 * if (sr_stream_next(stream, &not ) > 0)
 * {
 *     sr_print_notification(&not );
 * }
 * sr_stream_kill(stream);
 */
int sr_select_live(const struct sr_surreal_t *db,
                   sr_string_t *err_ptr,
                   struct sr_stream_t **stream_ptr,
                   const char *resource);

/**
 * merge data into existing records
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *merged;
 * sr_object_t *content = ...; // create content object
 * int len = sr_merge(db, &err, &merged, "foo:bar", content);
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * sr_free_arr(merged, len);
 * ```
 */
int sr_merge(const struct sr_surreal_t *db,
             sr_string_t *err_ptr,
             struct sr_value_t **res_ptr,
             const char *resource,
             const struct sr_object_t *content);

/**
 * Add a value at a JSON path using JSON Patch
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *patched;
 * sr_value_t value = ...; // create value to add
 * int len = sr_patch_add(db, &err, &patched, "person:john", "/tags/0", &value);
 * if (len < 0) {
 *     printf("Failed to patch: %s", err);
 *     return 1;
 * }
 * sr_free_arr(patched, len);
 * ```
 */
int sr_patch_add(const struct sr_surreal_t *db,
                 sr_string_t *err_ptr,
                 struct sr_value_t **res_ptr,
                 const char *resource,
                 const char *path,
                 const struct sr_value_t *value);

/**
 * Remove a value at a JSON path using JSON Patch
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *patched;
 * int len = sr_patch_remove(db, &err, &patched, "person:john", "/temporary_field");
 * if (len < 0) {
 *     printf("Failed to patch: %s", err);
 *     return 1;
 * }
 * sr_free_arr(patched, len);
 * ```
 */
int sr_patch_remove(const struct sr_surreal_t *db,
                    sr_string_t *err_ptr,
                    struct sr_value_t **res_ptr,
                    const char *resource,
                    const char *path);

/**
 * Replace a value at a JSON path using JSON Patch
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *patched;
 * sr_value_t value = ...; // create new value
 * int len = sr_patch_replace(db, &err, &patched, "person:john", "/name", &value);
 * if (len < 0) {
 *     printf("Failed to patch: %s", err);
 *     return 1;
 * }
 * sr_free_arr(patched, len);
 * ```
 */
int sr_patch_replace(const struct sr_surreal_t *db,
                     sr_string_t *err_ptr,
                     struct sr_value_t **res_ptr,
                     const char *resource,
                     const char *path,
                     const struct sr_value_t *value);

int sr_query(const struct sr_surreal_t *db,
             sr_string_t *err_ptr,
             struct sr_arr_res_t **res_ptr,
             const char *query,
             const struct sr_object_t *vars);

/**
 * select a resource
 *
 * can be used to select everything from a table or a single record
 * writes values to *res_ptr, and returns the number of values
 * result values are allocated by Surreal and must be freed with sr_free_arr
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *foos;
 * int len = sr_select(db, &err, &foos, "foo");
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 * for (int i = 0; i < len; i++)
 * {
 *     sr_value_print(&foos[i]);
 * }
 * sr_free_arr(foos, len);
 */
int sr_select(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource);

/**
 * set a variable for the current session
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *value = ...; // create value
 * if (sr_set(db, &err, "my_var", value) < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_set(const struct sr_surreal_t *db,
           sr_string_t *err_ptr,
           const char *key,
           const struct sr_value_t *value);

/**
 * Sign in utilizing the surreal authentication types.
 *
 * Used to provide credentials to a db for access permissions, either root or scoped.
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 *
 * sr_credentials_scope scope = sr_credentials_scope::ROOT;
 * const sr_string_t user = "<user>";
 * // SHOULD NEVER BE HARDCODED
 * const sr_string_t password = "<password>;
 * sr_credentials creds = sr_credentials {
 *     .username = user,
 *     .password = pass,
 * };
 *
 * if (sr_signin(db, &err, &scope, &creds, nullptr) < 0) {
 *     printf("Failed to authenticate credentials: %s", err);
 *     return 1;
 * }
 * ```
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 *
 * sr_credentials_scope scope = sr_credentials_scope::DATABASE;
 * const sr_string_t user = "<user>";
 * // SHOULD NEVER BE HARDCODED
 * const sr_string_t password = "<password>;
 * sr_credentials creds = sr_credentials {
 *     .username = user,
 *     .password = pass,
 * };
 * sr_string_t namespace_ = "testing";
 * sr_string_t db_name = "perf-test";
 * sr_credentials_access details = sr_credentials_access {
 *     .namespace_ = namespace_,
 *     .database = db_name,
 *     .access = nullptr,
 * };
 *
 * if (sr_signin(db, &err, &scope, &creds, &details) < 0) {
 *     printf("Failed to authenticate credentials: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_signin(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              const enum sr_credentials_scope *scope,
              const struct sr_credentials *creds,
              const struct sr_credentials_access *details);

/**
 * Sign up a new user with credentials
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_credentials_scope scope = sr_credentials_scope::RECORD;
 * const sr_string_t user = "newuser";
 * const sr_string_t password = "password123";
 * sr_credentials creds = sr_credentials {
 *     .username = user,
 *     .password = password,
 * };
 * sr_string_t namespace_ = "test";
 * sr_string_t db_name = "test";
 * sr_string_t access = "user";
 * sr_credentials_access details = sr_credentials_access {
 *     .namespace_ = namespace_,
 *     .database = db_name,
 *     .access = access,
 * };
 *
 * if (sr_signup(db, &err, &scope, &creds, &details) < 0) {
 *     printf("Failed to sign up: %s", err);
 *     return 1;
 * }
 * ```
 */
int sr_signup(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              const enum sr_credentials_scope *scope,
              const struct sr_credentials *creds,
              const struct sr_credentials_access *details);

/**
 * unset a variable from the current session
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_unset(db, &err, "my_var") < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_unset(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *key);

/**
 * update records with new content
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *updated;
 * sr_object_t *content = ...; // create content object
 * int len = sr_update(db, &err, &updated, "foo:bar", content);
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * sr_free_arr(updated, len);
 * ```
 */
int sr_update(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource,
              const struct sr_object_t *content);

/**
 * upsert (insert or update) records
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_value_t *upserted;
 * sr_object_t *content = ...; // create content object
 * int len = sr_upsert(db, &err, &upserted, "foo:bar", content);
 * if (len < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * sr_free_arr(upserted, len);
 * ```
 */
int sr_upsert(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource,
              const struct sr_object_t *content);

/**
 * select database
 * NOTE: namespace must be selected first with sr_use_ns
 *
 * # Examples
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_use_db(db, &err, "test") < 0)
 * {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_use_db(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *query);

/**
 * select namespace
 * NOTE: database must be selected before use with sr_use_db
 *
 * # Examples
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * if (sr_use_ns(db, &err, "test") < 0)
 * {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_use_ns(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *query);

/**
 * returns the db version
 * NOTE: version is allocated in Surreal and must be freed with sr_free_string
 * # Examples
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_string_t ver;
 *
 * if (sr_version(db, &err, &ver) < 0)
 * {
 *     printf("%s", err);
 *     return 1;
 * }
 * printf("%s", ver);
 * sr_free_string(ver);
 * ```
 */
int sr_version(const struct sr_surreal_t *db, sr_string_t *err_ptr, sr_string_t *res_ptr);

int sr_surreal_rpc_new(sr_string_t *err_ptr,
                       struct sr_surreal_rpc_t **surreal_ptr,
                       const char *endpoint,
                       struct sr_option_t options);

/**
 * execute rpc
 *
 * free result with sr_free_byte_arr
 */
int sr_surreal_rpc_execute(const struct sr_surreal_rpc_t *self,
                           sr_string_t *err_ptr,
                           uint8_t **res_ptr,
                           const uint8_t *ptr,
                           int len);

/**
 * Get a stream for receiving live query notifications
 *
 * Returns a stream that can be polled for notifications using sr_rpc_stream_next
 */
int sr_surreal_rpc_notifications(const struct sr_surreal_rpc_t *self,
                                 sr_string_t *err_ptr,
                                 struct sr_RpcStream **stream_ptr);

void sr_surreal_rpc_free(struct sr_surreal_rpc_t *ctx);

void sr_free_arr(struct sr_value_t *ptr, int len);

void sr_free_bytes(struct sr_bytes_t bytes);

void sr_free_byte_arr(uint8_t *ptr, int len);

void sr_print_notification(const struct sr_notification_t *notification);

const struct sr_value_t *sr_object_get(const struct sr_object_t *obj, const char *key);

struct sr_object_t sr_object_new(void);

void sr_object_insert(struct sr_object_t *obj, const char *key, const struct sr_value_t *value);

void sr_object_insert_str(struct sr_object_t *obj, const char *key, const char *value);

void sr_object_insert_int(struct sr_object_t *obj, const char *key, int value);

void sr_object_insert_float(struct sr_object_t *obj, const char *key, float value);

void sr_object_insert_double(struct sr_object_t *obj, const char *key, double value);

void sr_free_object(struct sr_object_t obj);

void sr_free_arr_res(struct sr_arr_res_t res);

void sr_free_arr_res_arr(struct sr_arr_res_t *ptr, int len);

/**
 * blocks until next item is recieved on stream
 * will return 1 and write notification to notification_ptr is recieved
 * will return SR_NONE if the stream is closed
 */
int sr_stream_next(struct sr_stream_t *self, struct sr_notification_t *notification_ptr);

void sr_stream_kill(struct sr_stream_t *stream);

/**
 * Get the next notification from the stream
 * Returns the length of the CBOR-encoded notification, or SR_CLOSED if the stream is closed
 */
int sr_rpc_stream_next(struct sr_RpcStream *self, uint8_t **res_ptr);

/**
 * Free an RpcStream
 */
void sr_rpc_stream_free(struct sr_RpcStream *stream);

void sr_free_string(sr_string_t string);

void sr_value_print(const struct sr_value_t *val);

bool sr_value_eq(const struct sr_value_t *lhs, const struct sr_value_t *rhs);

/**
 * Create a None value
 */
struct sr_value_t *sr_value_none(void);

/**
 * Create a Null value
 */
struct sr_value_t *sr_value_null(void);

/**
 * Create a Bool value
 */
struct sr_value_t *sr_value_bool(bool val);

/**
 * Create an Int value
 */
struct sr_value_t *sr_value_int(int64_t val);

/**
 * Create a Float value
 */
struct sr_value_t *sr_value_float(double val);

/**
 * Create a String value
 */
struct sr_value_t *sr_value_string(const char *val);

/**
 * Create an Object value from an existing object
 */
struct sr_value_t *sr_value_object(const struct sr_object_t *obj);

/**
 * Free a value created by sr_value_* functions
 */
void sr_value_free(struct sr_value_t *val);
