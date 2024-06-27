#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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

typedef struct StringResult {
  char *ok;
  struct SurrealError err;
} StringResult;

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

typedef enum Value_Tag {
  None,
  Null,
  Bool,
  Number,
  Strand,
  Duration,
} Value_Tag;

typedef struct Value {
  Value_Tag tag;
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
  };
} Value;

typedef struct Array {
  struct Value *arr;
  uintptr_t len;
} Array;

typedef struct ArrayResult {
  struct Array ok;
  struct SurrealError err;
} ArrayResult;

typedef struct ArrayResultArray {
  struct ArrayResult *arr;
  uintptr_t len;
} ArrayResultArray;

struct SurrealResult connect(const char *endpoint);

struct StringResult query(struct Surreal *db, const char *query);

char *select(struct Surreal *db, const char *resource);

void use_db(struct Surreal *db, const char *query);

void use_ns(struct Surreal *db, const char *query);

struct StringResult version(struct Surreal *db);

void free_arr_res(struct ArrayResult res);

void free_arr_res_arr(struct ArrayResultArray arr);

void free_string(char *string);

void free_arr(struct Array arr);
