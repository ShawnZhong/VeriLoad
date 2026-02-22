#include "nolibc.h"
#include "libbaz.h"
#include "libbar.h"

void libbaz_step(int depth) {
    write(1, "[libbaz] step\n", 14);
    if (depth <= 0) {
        write(1, "[libbaz] stop\n", 14);
        return;
    }

    libbar_step(depth - 1);
}

__attribute__((constructor))
static void libbaz_ctor(void) {
    write(1, "[libbaz] ctor\n", 14);
}

__attribute__((destructor))
static void libbaz_dtor(void) {
    write(1, "[libbaz] dtor\n", 14);
}
