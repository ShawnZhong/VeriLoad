#include <stdio.h>
#include "libfoo.h"
#include "libbar.h"

__attribute__((constructor))
static void main_ctor(void) {
    printf("[main] ctor\n");
}

int main(void) {
    printf("[main] entry\n");
    libfoo_print();
    libbar_step(3);
    printf("[main] exit\n");
    return 0;
}
