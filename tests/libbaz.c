#include "nolibc.h"

void libbaz_print(void) {
    (void)write(1, "[libbaz] function\n", 18);
}

__attribute__((constructor))
static void libbaz_ctor(void) {
    (void)write(1, "[libbaz] ctor\n", 14);
}
