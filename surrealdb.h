#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct BTreeMap_String__Value BTreeMap_String__Value;

typedef struct Surreal Surreal;

/**
 * when code = 0 there is no error
 *
 */
typedef struct SurrealError {
  int code;
  char *msg;
} SurrealError;

typedef struct SurrealResult {
  struct Surreal *ok;
  struct SurrealError err;
} SurrealResult;

typedef enum number_t_Tag {
  Int,
  Float,
} number_t_Tag;

typedef struct number_t {
  number_t_Tag tag;
  union {
    struct {
      int64_t int_;
    };
    struct {
      double float_;
    };
  };
} number_t;

typedef struct duration_t {
  uint64_t secs;
  uint32_t nanos;
} duration_t;

typedef struct uuid_t {
  uint8_t _0[16];
} uuid_t;

typedef struct object_t {
  struct BTreeMap_String__Value *_0;
} object_t;

typedef enum Id_Tag {
  IdNumber,
  IdString,
  IdArray,
  IdObject,
} Id_Tag;

typedef struct Id {
  Id_Tag tag;
  union {
    struct {
      int64_t id_number;
    };
    struct {
      char *id_string;
    };
    struct {
      struct array_t *id_array;
    };
    struct {
      struct object_t id_object;
    };
  };
} Id;

typedef struct thing_t {
  char *table;
  struct Id id;
} thing_t;

typedef enum value_t_Tag {
  None,
  Null,
  Bool,
  Number,
  Strand,
  Duration,
  Uuid,
  Array,
  Object,
  Thing,
} value_t_Tag;

typedef struct value_t {
  value_t_Tag tag;
  union {
    struct {
      bool bool_;
    };
    struct {
      struct number_t number;
    };
    struct {
      char *strand;
    };
    struct {
      struct duration_t duration;
    };
    struct {
      struct uuid_t uuid;
    };
    struct {
      struct array_t *array;
    };
    struct {
      struct object_t object;
    };
    struct {
      struct thing_t thing;
    };
  };
} value_t;

typedef struct array_t {
  struct value_t *arr;
  uintptr_t len;
} array_t;

typedef struct ArrayResult {
  struct array_t ok;
  struct SurrealError err;
} ArrayResult;

typedef struct ArrayResultArray {
  struct ArrayResult *arr;
  uintptr_t len;
} ArrayResultArray;

typedef struct ArrayResultArrayResult {
  struct ArrayResultArray ok;
  struct SurrealError err;
} ArrayResultArrayResult;

typedef struct StringResult {
  char *ok;
  struct SurrealError err;
} StringResult;

struct SurrealResult connect(const char *endpoint);

struct ArrayResultArrayResult query(struct Surreal *db, const char *query);

struct ArrayResult select(struct Surreal *db, const char *resource);

void use_db(struct Surreal *db, const char *query);

void use_ns(struct Surreal *db, const char *query);

struct StringResult version(struct Surreal *db);

void free_arr_res(struct ArrayResult res);

void free_arr_res_arr(struct ArrayResultArray arr);

void free_arr_res_arr_res(struct ArrayResultArrayResult res);

void free_string(char *string);

void free_arr(struct array_t arr);

const struct value_t *get(const struct object_t *obj, const char *key);

void print_value(const struct value_t *val);
