#include "nolibc.h"
#include "libbar.h"
#include "libbaz.h"

void libbar_step(int depth) {
    printf("[libbar] step=%d\n", depth);
    if (depth <= 0) {
        return;
    }

    libbaz_step(depth - 1);
}

__attribute__((constructor))
static void libbar_ctor(void) {
    printf("[libbar] ctor\n");
}

__attribute__((destructor))
static void libbar_dtor(void) {
    printf("[libbar] dtor\n");
}
