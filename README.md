# VeriLoad

[![CI](https://github.com/ShawnZhong/VeriLoad/actions/workflows/ci.yml/badge.svg)](https://github.com/ShawnZhong/VeriLoad/actions/workflows/ci.yml)

VeriLoad is a research prototype for verified ELF loading with Verus.

## Design Overview

- Input is a set of ELF objects (`main` + shared libraries) as raw bytes.
- The verified planner is stage-based: parse -> dependency discovery -> symbol resolution -> relocation and mapping plan.
- The planner produces `LoaderOutput` with mmap plans, relocation writes, entry PC, and constructor/destructor call order.
- Unverified runtime code executes that plan (`mmap`, memory writes, permission setup, constructor calls, jump to entry).
- Current scope focuses on ELF64/x86_64 with `R_X86_64_RELATIVE` and `R_X86_64_JUMP_SLOT`.

See `src/design.md` for the full design and refinement details.

## Quick Start

```bash
./install_verus.sh
./run.sh
```

`./run.sh` builds and loads a small dynamic-linked executable with the loader:

- `main` depends on `libfoo.so` and `libbar.so`.
- `libbar.so` and `libbaz.so` depend on each other with mutual recursion on `libbar_step` and `libbaz_step`.
- `libunused.so` is present in the input list but not needed by `main`, so it is parsed but not loaded/executed.
- Each loaded object has a constructor and destructor to be called.

<details>
<summary>Expected output</summary>

```text
[libbaz] ctor
[libbar] ctor
[libfoo] ctor
[main] ctor
[main] entry
[libfoo] function
[libbar] step=3
[libbaz] step=2
[libbar] step=1
[libbaz] step=0
[main] exit
[main] dtor
[libfoo] dtor
[libbar] dtor
[libbaz] dtor
```

</details>

<details>
<summary>Debug output</summary>

```text
entry_pc=0x000000000040651c
constructors=4
  ctor libbaz.so @ 0x000070000060657f
  ctor libbar.so @ 0x000070000040657f
  ctor libfoo.so @ 0x000070000020652b
  ctor main @ 0x00000000004064d4
destructors=4
  dtor main @ 0x00000000004064f8
  dtor libfoo.so @ 0x000070000020654f
  dtor libbar.so @ 0x00007000004065a3
  dtor libbaz.so @ 0x00007000006065a3
mmap_plans=16
  map main start=0x0000000000400000 len=4096 prot=R--
  ...
debug.reloc_writes=28
  ...
debug.parsed=5
  parsed[0] name=main elf_type=2 phdrs=5 needed=2 dynsyms=13 relas=0 jmprels=2
  ...
debug.discovered.order=[0, 1, 2, 3]
debug.resolved.planned=4
  planned[0] index=0 base=0x0000000000000000
  ...
debug.resolved.resolved_relocs=22
  resolved_reloc[0] requester=0 is_jmprel=true reloc_index=0 sym_index=1 provider_object=Some(2) provider_symbol=Some(11)
  ...
```

</details>
