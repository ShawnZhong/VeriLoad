#include "nolibc.h"

void libfoo_print(void) {
    write(1, "[libfoo] function\n", 18);
}

__attribute__((constructor))
static void libfoo_ctor(void) {
    write(1, "[libfoo] ctor\n", 14);
}
