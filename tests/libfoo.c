#include "nolibc.h"
#include "libfoo.h"

void libfoo_print(void) {
    write(1, "[libfoo] function\n", 18);
}

__attribute__((constructor))
static void libfoo_ctor(void) {
    write(1, "[libfoo] ctor\n", 14);
}

__attribute__((destructor))
static void libfoo_dtor(void) {
    write(1, "[libfoo] dtor\n", 14);
}
