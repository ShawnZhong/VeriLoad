#include "nolibc.h"
#include "libbar.h"
#include "libbaz.h"

void libbar_step(int depth) {
    write(1, "[libbar] step\n", 14);
    if (depth <= 0) {
        write(1, "[libbar] stop\n", 14);
        return;
    }

    libbaz_step(depth - 1);
}

__attribute__((constructor))
static void libbar_ctor(void) {
    write(1, "[libbar] ctor\n", 14);
}

__attribute__((destructor))
static void libbar_dtor(void) {
    write(1, "[libbar] dtor\n", 14);
}
