#include <stdio.h>
#include "../surrealdb.h"

int main()
{
    Surreal *db = connect("memory");

    char *ver_str = version(db);

    printf("%s", ver_str);
}