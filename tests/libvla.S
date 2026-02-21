.global foo
.type foo, @function
.extern bar
foo:
  call bar
  add $1, %eax
  ret
