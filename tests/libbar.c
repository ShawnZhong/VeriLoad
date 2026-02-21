#include "nolibc.h"

extern void libbaz_print(void);

void libbar_print(void) {
    write(1, "[libbar] function\n", 18);
    libbaz_print();
}

__attribute__((constructor))
static void libbar_ctor(void) {
    write(1, "[libbar] ctor\n", 14);
}
