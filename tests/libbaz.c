#include "nolibc.h"
#include "libbaz.h"

void libbaz_print(void) {
    write(1, "[libbaz] function\n", 18);
}

__attribute__((constructor))
static void libbaz_ctor(void) {
    write(1, "[libbaz] ctor\n", 14);
}

__attribute__((destructor))
static void libbaz_dtor(void) {
    write(1, "[libbaz] dtor\n", 14);
}
