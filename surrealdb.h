#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define sr_SR_ERROR -1

#define sr_SR_FATAL -2

#define sr_SR_NONE -3

typedef enum sr_action {
  SR_ACTION_CREATE,
  SR_ACTION_UPDATE,
  SR_ACTION_DELETE,
} sr_action;

typedef struct sr_opaque_object_internal_t sr_opaque_object_internal_t;

/**
 * may be sent across threads, but must not be aliased
 */
typedef struct sr_stream_t sr_stream_t;

typedef struct sr_surreal_t sr_surreal_t;

typedef char *sr_string_t;

typedef enum sr_number_t_Tag {
  SR_NUMBER_INT,
  SR_NUMBER_FLOAT,
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
  };
} sr_number_t;

typedef struct sr_duration_t {
  uint64_t secs;
  uint32_t nanos;
} sr_duration_t;

typedef struct sr_uuid_t {
  uint8_t _0[16];
} sr_uuid_t;

typedef struct sr_object_t {
  struct sr_opaque_object_internal_t *_0;
} sr_object_t;

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
      struct sr_bytes_t sr_value_bytes;
    };
    struct {
      struct sr_thing_t sr_value_thing;
    };
  };
} sr_value_t;

typedef struct sr_array_t {
  struct sr_value_t *arr;
  int len;
} sr_array_t;

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

typedef struct sr_notification_t {
  bool some;
  struct sr_uuid_t query_id;
  enum sr_action action;
  struct sr_value_t data;
} sr_notification_t;

int sr_connect(sr_string_t *err_ptr, struct sr_surreal_t **surreal_ptr, const char *endpoint);

void sr_surreal_disconnect(struct sr_surreal_t *db);

/**
 * if successful sets *stream_ptr to be an exclusive reference to an opaque Stream object
 * this pointer should not be copied and only one should be used at a time
 */
int sr_select_live(const struct sr_surreal_t *db,
                   sr_string_t *err_ptr,
                   struct sr_stream_t **stream_ptr,
                   const char *resource);

int sr_query(const struct sr_surreal_t *db,
             sr_string_t *err_ptr,
             struct sr_arr_res_t **res_ptr,
             const char *query);

int sr_select(const struct sr_surreal_t *db,
              sr_string_t *err_ptr,
              struct sr_value_t **res_ptr,
              const char *resource);

int sr_use_db(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *query);

int sr_use_ns(const struct sr_surreal_t *db, sr_string_t *err_ptr, const char *query);

int sr_version(const struct sr_surreal_t *db, sr_string_t *err_ptr, sr_string_t *res_ptr);

void sr_free_arr(struct sr_value_t *ptr, int len);

void sr_free_bytes(struct sr_bytes_t bytes);

void sr_print_notification(const struct sr_notification_t *notification);

const struct sr_value_t *sr_object_get(const struct sr_object_t *obj, const char *key);

void sr_free_object(struct sr_object_t obj);

void sr_free_arr_res(struct sr_arr_res_t res);

void sr_free_arr_res_arr(struct sr_arr_res_t *ptr, int len);

struct sr_notification_t sr_stream_next(struct sr_stream_t *self);

void sr_stream_kill(struct sr_stream_t *stream);

void sr_free_string(sr_string_t string);

void sr_value_print(const struct sr_value_t *val);
