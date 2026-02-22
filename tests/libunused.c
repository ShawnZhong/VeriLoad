#include "nolibc.h"

void libunused_print(void) {
    write(1, "[libunused] function\n", 21);
}

__attribute__((constructor))
static void libunused_ctor(void) {
    write(1, "[libunused] ctor\n", 17);
}

__attribute__((destructor))
static void libunused_dtor(void) {
    write(1, "[libunused] dtor\n", 17);
}
