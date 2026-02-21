#include "nolibc.h"

extern void libfoo_print(void);
extern void libbar_print(void);

void main_print(void) {
    write(1, "[main] function\n", 16);
}

__attribute__((constructor))
static void main_ctor(void) {
    write(1, "[main] ctor\n", 12);
}

void _start(void) {
    write(1, "[main] entry\n", 13);
    main_print();
    libfoo_print();
    libbar_print();
    write(1, "PASS\n", 5);
    exit(0);
}
