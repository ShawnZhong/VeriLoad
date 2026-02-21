#ifndef VERILOAD_NO_LIBC_ENTRY_H
#define VERILOAD_NO_LIBC_ENTRY_H

static __attribute__((noreturn)) void exit_with_code(int code) {
    __asm__ volatile(
        "mov $60, %%rax\n\t"
        "mov %0, %%rdi\n\t"
        "syscall\n\t"
        :
        : "r"((long)code)
        : "rax", "rdi", "rcx", "r11", "memory"
    );
    __builtin_unreachable();
}

#endif
