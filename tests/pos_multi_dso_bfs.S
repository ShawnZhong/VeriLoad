.global _start
.type _start, @function
.extern foo
_start:
  call foo
  mov %eax, %edi
  mov $60, %rax
  syscall
