#include "nolibc.h"
#include "libbaz.h"
#include "libbar.h"

void libbaz_step(int depth) {
    printf("[libbaz] step=%d\n", depth);
    if (depth <= 0) {
        return;
    }

    libbar_step(depth - 1);
}

__attribute__((constructor))
static void libbaz_ctor(void) {
    printf("[libbaz] ctor\n");
}

__attribute__((destructor))
static void libbaz_dtor(void) {
    printf("[libbaz] dtor\n");
}
