#include <stdio.h>

void libunused_print(void) {
    printf("[libunused] function\n");
}

__attribute__((constructor))
static void libunused_ctor(void) {
    printf("[libunused] ctor\n");
}

__attribute__((destructor))
static void libunused_dtor(void) {
    printf("[libunused] dtor\n");
}
