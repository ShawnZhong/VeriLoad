#include "nolibc.h"
#include "libbar.h"
#include "libbaz.h"

void libbar_print(void) {
    write(1, "[libbar] function\n", 18);
    libbaz_print();
}

__attribute__((constructor))
static void libbar_ctor(void) {
    write(1, "[libbar] ctor\n", 14);
}

__attribute__((destructor))
static void libbar_dtor(void) {
    write(1, "[libbar] dtor\n", 14);
}
