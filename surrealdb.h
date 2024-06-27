#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Surreal Surreal;

struct Surreal *connect(const char *endpoint);

char *version(struct Surreal *db);
