#include "no_libc_entry.h"

void _start(void) {
    exit_with_code(0);
}
