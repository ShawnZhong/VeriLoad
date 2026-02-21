#include "nolibc.h"
#include "libfoo.h"
#include "libbar.h"

void main_print(void) {
    write(1, "[main] function\n", 16);
}

__attribute__((constructor))
static void main_ctor(void) {
    write(1, "[main] ctor\n", 12);
}

__attribute__((destructor))
static void main_dtor(void) {
    write(1, "[main] dtor\n", 12);
}

int main(void) {
    write(1, "[main] entry\n", 13);
    main_print();
    libfoo_print();
    libbar_print();
    return 0;
}
