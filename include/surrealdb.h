#pragma once

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
  SR_ACTION_KILLED,
} sr_action;

typedef struct sr_opaque_object_internal_t sr_opaque_object_internal_t;

/**
 * Stream for receiving RPC live query notifications
 *
 * Wraps a `Receiver<PublicNotification>` from the datastore's notification channel.
 * Uses synchronous blocking receives, so no async drop is required.
 */
typedef struct sr_RpcStream sr_RpcStream;

/**
 * Stream for receiving live query notifications
 *
 * May be sent across threads, but must not be aliased.
 * Use `sr_stream_next` to receive notifications and `sr_stream_kill` to close.
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
 * The object representing a Surreal RPC connection
 *
 * It is safe to be referenced from multiple threads
 * If any operation, on any thread returns SR_FATAL then the connection is poisoned and must not be used again.
 * (use will cause the program to abort)
 *
 * should be freed with sr_surreal_rpc_free
 */
typedef struct sr_surreal_rpc_t sr_surreal_rpc_t;

/**
 * A null-terminated C string type
 *
 * This is a wrapper around a raw C string pointer that handles memory management.
 * Strings returned by SurrealDB functions must be freed with `sr_free_string`.
 */
typedef char *sr_string_t;

/**
 * A key-value object type for SurrealDB
 *
 * Contains string keys mapped to Value instances.
 */
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

/**
 * Represents a SurrealDB value
 *
 * This enum wraps all possible value types that can be returned from SurrealDB queries
 * or used as input parameters. Each variant corresponds to a SurrealDB data type.
 */
typedef enum sr_value_t_Tag {
  /**
   * No value (absence of data)
   */
  SR_VALUE_NONE,
  /**
   * Explicit null value
   */
  SR_VALUE_NULL,
  /**
   * Boolean value (true/false)
   */
  SR_VALUE_BOOL,
  /**
   * Numeric value (integer, float, or decimal)
   */
  SR_VALUE_NUMBER,
  /**
   * String value
   */
  SR_VALUE_STRAND,
  /**
   * Duration value
   */
  SR_VALUE_DURATION,
  /**
   * DateTime value in RFC3339 format
   */
  SR_VALUE_DATETIME,
  /**
   * UUID value
   */
  SR_VALUE_UUID,
  /**
   * Array of values
   */
  SR_VALUE_ARRAY,
  /**
   * Object (key-value map)
   */
  SR_VALUE_OBJECT,
  /**
   * Geometry object (points, lines, polygons, etc.)
   */
  SR_GEOMETRY_OBJECT,
  /**
   * Raw bytes
   */
  SR_VALUE_BYTES,
  /**
   * Record ID (thing)
   */
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

/**
 * Connection options for SurrealDB
 *
 * Configures various settings for the database connection.
 */
typedef struct sr_option_t {
  /**
   * Enable strict mode for queries
   */
  bool strict;
  /**
   * Query timeout in seconds
   */
  uint8_t query_timeout;
  /**
   * Transaction timeout in seconds
   */
  uint8_t transaction_timeout;
} sr_option_t;

typedef struct sr_notification_t {
  struct sr_uuid_t query_id;
  enum sr_action action;
  struct sr_value_t data;
} sr_notification_t;

/**
 * Connects to a local, remote, or embedded database
 *
 * If any function returns SR_FATAL, the connection is poisoned and must not be used
 * (except to drop). Continued use will cause the program to abort.
 *
 * # Safety
 *
 * - `err_ptr` must be a valid pointer or null (errors ignored if null)
 * - `surreal_ptr` must be a valid pointer to receive the connection handle
 * - `endpoint` must be a valid null-terminated UTF-8 string
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
int sr_connect(sr_string_t *err_ptr, struct sr_surreal_t **surreal_ptr, const char *endpoint);

/**
 * Disconnect a database connection
 *
 * The Surreal object must not be used after this function has been called.
 * Any object allocations will still be valid and should be freed using the appropriate function.
 *
 * Note: Stream objects should be killed before disconnection to ensure proper cleanup.
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
 * Clone a database connection
 *
 * Creates a new independent Surreal handle that shares the same underlying
 * database engine as the original. The cloned handle has its own Tokio runtime
 * and independent namespace/database state.
 *
 * This is equivalent to Rust SDK's `db.clone()` — enabling multiple independent
 * sessions on a single embedded (surrealkv://) connection without file locking issues.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `clone_ptr` must be a valid pointer to receive the cloned handle
 * - The original `db` must NOT be disconnected while clones are in use
 * - Each clone must be freed with `sr_surreal_disconnect`
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_surreal_t *clone;
 * sr_string_t err;
 *
 * // Connect once
 * if (sr_connect(&err, &db, "surrealkv://test.skv") < 0) {
 *     printf("error: %s\n", err);
 *     return 1;
 * }
 *
 * // Clone for independent session
 * if (sr_clone(db, &err, &clone) < 0) {
 *     printf("error: %s\n", err);
 *     return 1;
 * }
 *
 * // Each handle can use different NS/DB independently
 * sr_use_ns(db, &err, "ns1");
 * sr_use_db(db, &err, "db1");
 * sr_use_ns(clone, &err, "ns2");
 * sr_use_db(clone, &err, "db2");
 *
 * // Queries run concurrently on separate sessions
 * // ...
 *
 * sr_surreal_disconnect(clone);
 * sr_surreal_disconnect(db);
 * ```
 */
int sr_clone(const struct sr_surreal_t *db, sr_string_t *err_ptr, struct sr_surreal_t **clone_ptr);

/**
 * Authenticate with a token
 *
 * Authenticates the current connection with a JWT token.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `token` must be a valid null-terminated UTF-8 string
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
 * Begin a new transaction
 *
 * Starts a new database transaction.
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
 * Cancel the current transaction
 *
 * Cancels and rolls back the current database transaction.
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
 * Commit the current transaction
 *
 * Commits and finalizes the current database transaction.
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
 * Create a record
 *
 * Creates a new record in the specified resource with the given content.
 * The resource can be a table name (e.g., "user") for auto-generated IDs,
 * or a specific record ID (e.g., "user:john").
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` may be null (result will be discarded)
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
 */
int sr_create(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_object_t **res_ptr,
              const char *resource,
              const struct sr_object_t *content);

/**
 * Delete a record or records
 *
 * Deletes records from the specified resource.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
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
 * Export database data to a file
 *
 * Exports all data from the current namespace and database to a file.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `file_path` must be a valid null-terminated UTF-8 string
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
 * Check the health of the database server
 *
 * Performs a health check on the database connection.
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
 * Import database data from a file
 *
 * Imports data from a file into the current namespace and database.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `file_path` must be a valid null-terminated UTF-8 string
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
 * Insert one or more records
 *
 * Inserts records into the specified resource with the given content.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
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
 * Creates a relation record in a relation table.
 *
 * The content object must contain 'in' and 'out' fields specifying the records to relate.
 * Additional fields can be added as relation properties.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `table` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
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
 * Runs a custom or built-in SurrealDB function with the specified arguments.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `function_name` must be a valid null-terminated UTF-8 string
 * - `args` may be null (empty arguments)
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
 * Establishes a directed relation from one record to another through a relation table.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `from` must be a valid null-terminated UTF-8 string
 * - `relation` must be a valid null-terminated UTF-8 string
 * - `to` must be a valid null-terminated UTF-8 string
 * - `content` may be null (no content will be added)
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
 * Invalidate the current authentication session
 *
 * Clears the current authentication for the connection.
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
 * Kill a live query by its UUID string
 *
 * Terminates an active live query subscription.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `query_id` must be a valid null-terminated UTF-8 string
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * const char *query_id = "..."; // UUID string from live query
 * if (sr_kill(db, &err, query_id) < 0) {
 *     printf("%s", err);
 *     return 1;
 * }
 * ```
 */
int sr_kill(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *query_id);

/**
 * Make a live selection
 *
 * Creates a live query subscription that streams changes.
 * if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
 * which can be moved across threads but not aliased
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `stream_ptr` must be a valid pointer to receive the stream
 * - `resource` must be a valid null-terminated UTF-8 string
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
 * Merge data into existing records
 *
 * Merges the provided content into existing records, preserving unmodified fields.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
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
 * Applies a JSON Patch add operation to the specified resource.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `path` must be a valid null-terminated UTF-8 string
 * - `value` must be a valid pointer to a Value
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
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `path` must be a valid null-terminated UTF-8 string
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
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `path` must be a valid null-terminated UTF-8 string
 * - `value` must be a valid pointer to a Value
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

/**
 * Execute a SurrealQL query
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `query` must be a valid null-terminated UTF-8 string
 * - `vars` may be null (no variables bound)
 */
int sr_query(const struct sr_surreal_t *db,
             sr_string_t *err_ptr,
             struct sr_arr_res_t **res_ptr,
             const char *query,
             const struct sr_object_t *vars);

/**
 * Select a resource
 *
 * Selects records from the specified resource (table or record ID).
 *
 * can be used to select everything from a table or a single record
 * writes values to *res_ptr, and returns the number of values
 * result values are allocated by Surreal and must be freed with sr_free_arr
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
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
 * Set a variable for the current session
 *
 * Defines a session variable that can be referenced in queries.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `key` must be a valid null-terminated UTF-8 string
 * - `value` must be a valid pointer to a Value
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
 * Sign in utilizing the surreal authentication types
 *
 * Authenticates with the database using the provided credentials.
 *
 * Used to provide credentials to a db for access permissions, either root or scoped.
 * Returns the JWT token via token_ptr if not null.
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_string_t token;
 *
 * sr_credentials_scope scope = sr_credentials_scope::ROOT;
 * const sr_string_t user = "<user>";
 * // SHOULD NEVER BE HARDCODED
 * const sr_string_t password = "<password>";
 * sr_credentials creds = sr_credentials {
 *     .username = user,
 *     .password = pass,
 * };
 *
 * if (sr_signin(db, &err, &token, &scope, &creds, nullptr, nullptr) < 0) {
 *     printf("Failed to authenticate credentials: %s", err);
 *     return 1;
 * }
 * // token now contains the JWT
 * sr_free_string(token);
 * ```
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_string_t token;
 *
 * sr_credentials_scope scope = sr_credentials_scope::DATABASE;
 * const sr_string_t user = "<user>";
 * // SHOULD NEVER BE HARDCODED
 * const sr_string_t password = "<password>";
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
 * if (sr_signin(db, &err, &token, &scope, &creds, &details, nullptr) < 0) {
 *     printf("Failed to authenticate credentials: %s", err);
 *     return 1;
 * }
 * ```
 * For RECORD scope with custom params:
 * ```c
 * sr_object_t params = sr_object_new();
 * sr_object_insert_str(&params, "email", "user@example.com");
 * sr_object_insert_str(&params, "password", "secret");
 * if (sr_signin(db, &err, &token, &scope, nullptr, &details, &params) < 0) {
 *     // handle error
 * }
 * ```
 */
int sr_signin(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              sr_string_t *token_ptr,
              const enum sr_credentials_scope *scope,
              const struct sr_credentials *creds,
              const struct sr_credentials_access *details,
              const struct sr_object_t *params);

/**
 * Sign up a new user with credentials
 *
 * Registers a new user account with the database.
 * Returns the JWT token via token_ptr if not null.
 * Only RECORD scope is supported for signup.
 *
 * # Examples
 *
 * ```c
 * sr_surreal_t *db;
 * sr_string_t err;
 * sr_string_t token;
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
 * if (sr_signup(db, &err, &token, &scope, &creds, &details, nullptr) < 0) {
 *     printf("Failed to sign up: %s", err);
 *     return 1;
 * }
 * // token now contains the JWT
 * sr_free_string(token);
 * ```
 * For custom params:
 * ```c
 * sr_object_t params = sr_object_new();
 * sr_object_insert_str(&params, "email", "user@example.com");
 * sr_object_insert_str(&params, "password", "secret");
 * if (sr_signup(db, &err, &token, &scope, nullptr, &details, &params) < 0) {
 *     // handle error
 * }
 * ```
 */
int sr_signup(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              sr_string_t *token_ptr,
              const enum sr_credentials_scope *scope,
              const struct sr_credentials *creds,
              const struct sr_credentials_access *details,
              const struct sr_object_t *params);

/**
 * Unset a variable from the current session
 *
 * Removes a previously defined session variable.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `key` must be a valid null-terminated UTF-8 string
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
 * Update records with new content
 *
 * Replaces the content of existing records with new data.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
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
 * Upsert (insert or update) records
 *
 * Creates records if they don't exist, or updates them if they do.
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result array
 * - `resource` must be a valid null-terminated UTF-8 string
 * - `content` must be a valid pointer to an Object
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
 * Select database
 *
 * Sets the database to use for subsequent operations.
 * NOTE: namespace must be selected first with sr_use_ns
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `db_name` must be a valid null-terminated UTF-8 string
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
int sr_use_db(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *db_name);

/**
 * Select namespace
 *
 * Sets the namespace to use for subsequent operations.
 * NOTE: database must be selected before use with sr_use_db
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `ns_name` must be a valid null-terminated UTF-8 string
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
int sr_use_ns(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *ns_name);

/**
 * Returns the database version
 *
 * Retrieves the version string of the connected SurrealDB server.
 * NOTE: version is allocated in Surreal and must be freed with sr_free_string
 *
 * # Safety
 *
 * - `db` must be a valid pointer to a Surreal connection
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the version string
 *
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
 * Execute an RPC request via raw CBOR bytes
 *
 * # Safety
 *
 * - `err_ptr` must be a valid pointer or null
 * - `res_ptr` must be a valid pointer to receive the result
 * - `ptr` must be a valid pointer to CBOR-encoded request data
 * - `len` must be the length of the data at ptr
 *
 * Free result with sr_free_byte_arr
 */
int sr_surreal_rpc_execute(const struct sr_surreal_rpc_t *self,
                           sr_string_t *err_ptr,
                           uint8_t **res_ptr,
                           const uint8_t *ptr,
                           int len);

/**
 * Get a stream for receiving live query notifications
 *
 * # Safety
 *
 * - `err_ptr` must be a valid pointer or null
 * - `stream_ptr` must be a valid pointer to receive the stream
 *
 * Returns a stream that can be polled for notifications using sr_rpc_stream_next
 */
int sr_surreal_rpc_notifications(const struct sr_surreal_rpc_t *self,
                                 sr_string_t *err_ptr,
                                 struct sr_RpcStream **stream_ptr);

/**
 * Free an RPC context
 */
void sr_surreal_rpc_free(struct sr_surreal_rpc_t *ctx);

void sr_free_arr(struct sr_value_t *ptr, int len);

/**
 * Get the length of an array
 */
int sr_array_len(const struct sr_array_t *arr);

/**
 * Get a value at the specified index (returns NULL if out of bounds)
 * The returned pointer is borrowed and should NOT be freed by the caller
 */
const struct sr_value_t *sr_array_get(const struct sr_array_t *arr, int index);

/**
 * Create a new array with the given value appended
 * Returns a new array - the original array is not modified
 * The caller is responsible for freeing the returned array
 */
struct sr_array_t *sr_array_push(const struct sr_array_t *arr, const struct sr_value_t *value);

/**
 * Free an array created by sr_array_push
 */
void sr_array_free(struct sr_array_t *arr);

void sr_free_bytes(struct sr_bytes_t bytes);

void sr_free_byte_arr(uint8_t *ptr, int len);

void sr_print_notification(const struct sr_notification_t *notification);

/**
 * Get a value by key from the object
 *
 * # Safety
 *
 * - `obj` must be a valid reference to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 */
const struct sr_value_t *sr_object_get(const struct sr_object_t *obj, const char *key);

/**
 * Create a new empty object
 */
struct sr_object_t sr_object_new(void);

/**
 * Insert a key-value pair into the object
 *
 * # Safety
 *
 * - `obj` must be a valid pointer to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 * - `value` must be a valid reference to a Value
 *
 * If any pointer is null, the function returns without modification.
 */
void sr_object_insert(struct sr_object_t *obj, const char *key, const struct sr_value_t *value);

/**
 * Insert a string value into the object
 *
 * # Safety
 *
 * - `obj` must be a valid pointer to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 * - `value` must be a valid null-terminated UTF-8 string
 */
void sr_object_insert_str(struct sr_object_t *obj, const char *key, const char *value);

/**
 * Insert an integer value into the object
 *
 * # Safety
 *
 * - `obj` must be a valid pointer to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 */
void sr_object_insert_int(struct sr_object_t *obj, const char *key, int value);

/**
 * Insert a float value into the object
 *
 * # Safety
 *
 * - `obj` must be a valid pointer to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 */
void sr_object_insert_float(struct sr_object_t *obj, const char *key, float value);

/**
 * Insert a double value into the object
 *
 * # Safety
 *
 * - `obj` must be a valid pointer to an Object
 * - `key` must be a valid null-terminated UTF-8 string
 */
void sr_object_insert_double(struct sr_object_t *obj, const char *key, double value);

/**
 * Free an object
 */
void sr_free_object(struct sr_object_t obj);

/**
 * Get the number of key-value pairs in the object
 */
int sr_object_len(const struct sr_object_t *obj);

/**
 * Get all keys from the object as a null-terminated array of strings
 * Returns the number of keys, or -1 on error
 * The caller must free the returned array using sr_free_string_arr
 */
int sr_object_keys(const struct sr_object_t *obj, char ***keys_ptr);

/**
 * Free a string array returned by sr_object_keys
 */
void sr_free_string_arr(char **arr, int len);

void sr_free_arr_res(struct sr_arr_res_t res);

void sr_free_arr_res_arr(struct sr_arr_res_t *ptr, int len);

/**
 * Blocks until next item is received on stream
 * will return 1 and write notification to notification_ptr if received
 * will return SR_NONE if the stream is closed
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
int sr_stream_next(struct sr_stream_t *self, struct sr_notification_t *notification_ptr);

/**
 * Kill and free a stream
 *
 * Closes the stream and releases all associated resources.
 * The stream must not be used after calling this function.
 */
void sr_stream_kill(struct sr_stream_t *stream);

/**
 * Get the next notification from the stream
 *
 * Returns the length of the CBOR-encoded notification, or SR_CLOSED if the
 * channel is closed. The CBOR-encoded bytes are written to *res_ptr.
 *
 * Free the result with sr_free_byte_arr.
 */
int sr_rpc_stream_next(struct sr_RpcStream *self, uint8_t **res_ptr);

/**
 * Free an RpcStream
 */
void sr_rpc_stream_free(struct sr_RpcStream *stream);

/**
 * Free a string allocated by SurrealDB
 *
 * This function must be called to free strings returned by SurrealDB functions
 * to avoid memory leaks.
 */
void sr_free_string(sr_string_t string);

/**
 * Print a value to stdout for debugging
 *
 * Outputs the debug representation of the value to standard output.
 */
void sr_value_print(const struct sr_value_t *val);

/**
 * Compare two values for equality
 *
 * Returns true if both values are equal, false otherwise.
 */
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
 * Create a Duration value
 */
struct sr_value_t *sr_value_duration(uint64_t secs, uint32_t nanos);

/**
 * Create a Datetime value from RFC3339 string (e.g. "2024-01-15T10:30:00Z")
 */
struct sr_value_t *sr_value_datetime(const char *val);

/**
 * Create a UUID value from 16 bytes
 */
struct sr_value_t *sr_value_uuid(const uint8_t *bytes);

/**
 * Create an empty Array value
 */
struct sr_value_t *sr_value_array(void);

/**
 * Create a Bytes value from raw data
 */
struct sr_value_t *sr_value_bytes(const uint8_t *data, int len);

/**
 * Create a Thing value (record ID) from table name and string ID
 */
struct sr_value_t *sr_value_thing(const char *table, const char *id);

/**
 * Free a value created by sr_value_* functions
 */
void sr_value_free(struct sr_value_t *val);

/**
 * Create a Point geometry value
 */
struct sr_value_t *sr_value_point(double x, double y);

/**
 * Create a LineString geometry value from an array of coordinates
 * coords is a pointer to an array of sr_g_coord structures
 */
struct sr_value_t *sr_value_linestring(const struct sr_sr_g_coord *coords, int len);

/**
 * Create a simple Polygon geometry value from exterior ring coordinates
 * coords is a pointer to an array of sr_g_coord structures for the exterior ring
 */
struct sr_value_t *sr_value_polygon(const struct sr_sr_g_coord *coords, int len);

/**
 * Create a MultiPoint geometry value from an array of points (x,y pairs)
 * coords is a pointer to an array of sr_g_coord structures
 */
struct sr_value_t *sr_value_multipoint(const struct sr_sr_g_coord *coords, int len);

/**
 * Create a MultiLineString geometry value
 * linestrings is an array of pointers to coordinate arrays
 * lens is an array of lengths for each linestring
 * count is the number of linestrings
 */
struct sr_value_t *sr_value_multilinestring(const struct sr_sr_g_coord *const *linestrings,
                                            const int *lens,
                                            int count);

/**
 * Create a MultiPolygon geometry value
 * polygons is an array of pointers to coordinate arrays (exterior rings only)
 * lens is an array of lengths for each polygon's exterior ring
 * count is the number of polygons
 */
struct sr_value_t *sr_value_multipolygon(const struct sr_sr_g_coord *const *polygons,
                                         const int *lens,
                                         int count);

/**
 * Create a Decimal value from string representation
 */
struct sr_value_t *sr_value_decimal(const char *val);
