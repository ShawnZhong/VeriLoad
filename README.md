# VeriLoad

[![CI](https://github.com/ShawnZhong/VeriLoad/actions/workflows/ci.yml/badge.svg)](https://github.com/ShawnZhong/VeriLoad/actions/workflows/ci.yml)

VeriLoad is a research prototype for a verified ELF loader in Verus.

## Design Overview

VeriLoad runs in three steps:
1. Unverified input setup: read the provided ELF objects (`main` + shared libraries) into `LoaderInput`.
2. Verified planner: `parse -> discover -> resolve -> mmap_plan -> relocate_plan -> relocate_apply -> final`, producing `LoaderOutput`.
3. Unverified runtime: execute `LoaderOutput` (`mmap`, copy bytes, set permissions, call constructors, jump to entry, call destructors).

See [`design.md`](design.md) for the full design and refinement details.

## Quick Start

```bash
git clone --recursive https://github.com/ShawnZhong/VeriLoad.git
cd VeriLoad
./install_deps.sh
make
./run.sh
```

`./run.sh` builds the loader and runs a dynamic-linked executable with it (see [`tests/main.c`](tests/main.c)):

- `main` depends on `libfoo.so` and `libbar.so`. It calls `libfoo_print` and `libbar_step`.
- `libbar.so` and `libbaz.so` depend on each other with mutual recursion on `libbar_step` and `libbaz_step`.
- Each loaded object has a constructor and destructor to be called.

Expected output:
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
```


Debug output: `./run.sh --debug`

```text
entry_pc=0x00000000004064ee
constructors=4
  ctor libbaz.so @ 0x0000700000606567
  ctor libbar.so @ 0x0000700000406567
  ctor libfoo.so @ 0x0000700000206526
  ctor main @ 0x00000000004064b0
destructors=4
  dtor main @ 0x00000000004064cf
  dtor libfoo.so @ 0x0000700000206545
  dtor libbar.so @ 0x0000700000406586
  dtor libbaz.so @ 0x0000700000606586
mmap_plans=16
  map main start=0x0000000000400000 len=4096 prot=R--
  map main start=0x0000000000401000 len=24576 prot=R-X
  map main start=0x0000000000407000 len=12288 prot=R--
  map main start=0x000000000040a000 len=8192 prot=RW-
  map libfoo.so start=0x0000700000200000 len=4096 prot=R--
  map libfoo.so start=0x0000700000201000 len=24576 prot=R-X
  map libfoo.so start=0x0000700000207000 len=12288 prot=R--
  map libfoo.so start=0x000070000020a000 len=8192 prot=RW-
  map libbar.so start=0x0000700000400000 len=4096 prot=R--
  map libbar.so start=0x0000700000401000 len=24576 prot=R-X
  map libbar.so start=0x0000700000407000 len=12288 prot=R--
  map libbar.so start=0x000070000040a000 len=8192 prot=RW-
  map libbaz.so start=0x0000700000600000 len=4096 prot=R--
  map libbaz.so start=0x0000700000601000 len=24576 prot=R-X
  map libbaz.so start=0x0000700000607000 len=12288 prot=R--
  map libbaz.so start=0x000070000060a000 len=8192 prot=RW-
debug.reloc_writes=28
  reloc libfoo.so addr=0x000070000020ae48 value=0x0000700000206526 type=8
  reloc libfoo.so addr=0x000070000020ae50 value=0x0000700000206545 type=8
  reloc libbar.so addr=0x000070000040ae38 value=0x0000700000406567 type=8
  reloc libbar.so addr=0x000070000040ae40 value=0x0000700000406586 type=8
  reloc libbaz.so addr=0x000070000060ae38 value=0x0000700000606567 type=8
  reloc libbaz.so addr=0x000070000060ae40 value=0x0000700000606586 type=8
  reloc main addr=0x000000000040b000 value=0x0000700000406527 type=7
  reloc main addr=0x000000000040b008 value=0x0000700000206507 type=7
  reloc libfoo.so addr=0x000070000020afc8 value=0x000000000040b028 type=6
  reloc libfoo.so addr=0x000070000020afd0 value=0x000000000040b034 type=6
  reloc libfoo.so addr=0x000070000020afd8 value=0x000000000040b030 type=6
  reloc libfoo.so addr=0x000070000020afe0 value=0x000000000040b038 type=6
  reloc libfoo.so addr=0x000070000020b000 value=0x0000000000406534 type=7
  reloc libfoo.so addr=0x000070000020b008 value=0x0000000000406584 type=7
  reloc libbar.so addr=0x000070000040afc8 value=0x000000000040b028 type=6
  reloc libbar.so addr=0x000070000040afd0 value=0x000000000040b034 type=6
  reloc libbar.so addr=0x000070000040afd8 value=0x000000000040b030 type=6
  reloc libbar.so addr=0x000070000040afe0 value=0x000000000040b038 type=6
  reloc libbar.so addr=0x000070000040b000 value=0x0000700000606527 type=7
  reloc libbar.so addr=0x000070000040b008 value=0x0000000000406534 type=7
  reloc libbar.so addr=0x000070000040b010 value=0x0000000000406584 type=7
  reloc libbaz.so addr=0x000070000060afc8 value=0x000000000040b028 type=6
  reloc libbaz.so addr=0x000070000060afd0 value=0x000000000040b034 type=6
  reloc libbaz.so addr=0x000070000060afd8 value=0x000000000040b030 type=6
  reloc libbaz.so addr=0x000070000060afe0 value=0x000000000040b038 type=6
  reloc libbaz.so addr=0x000070000060b000 value=0x0000000000406534 type=7
  reloc libbaz.so addr=0x000070000060b008 value=0x0000700000406527 type=7
  reloc libbaz.so addr=0x000070000060b010 value=0x0000000000406584 type=7
debug.parsed=5
  parsed[0] name=main elf_type=2 phdrs=5 needed=2 dynsyms=13 relas=0 jmprels=2
  parsed[1] name=libfoo.so elf_type=3 phdrs=5 needed=0 dynsyms=12 relas=6 jmprels=2
  parsed[2] name=libbar.so elf_type=3 phdrs=5 needed=1 dynsyms=13 relas=6 jmprels=3
  parsed[3] name=libbaz.so elf_type=3 phdrs=5 needed=1 dynsyms=13 relas=6 jmprels=3
  parsed[4] name=libunused.so elf_type=3 phdrs=5 needed=0 dynsyms=12 relas=6 jmprels=2
debug.discovered.order=[0, 1, 2, 3]
debug.resolved.planned=4
  planned[0] index=0 base=0x0000000000000000
  planned[1] index=1 base=0x0000000000000000
  planned[2] index=2 base=0x0000000000000000
  planned[3] index=3 base=0x0000000000000000
debug.resolved.resolved_relocs=22
  resolved_reloc[0] requester=0 is_jmprel=true reloc_index=0 sym_index=1 provider_object=Some(2) provider_symbol=Some(11)
  resolved_reloc[1] requester=0 is_jmprel=true reloc_index=1 sym_index=2 provider_object=Some(1) provider_symbol=Some(3)
  resolved_reloc[2] requester=1 is_jmprel=false reloc_index=2 sym_index=10 provider_object=Some(0) provider_symbol=Some(11)
  resolved_reloc[3] requester=1 is_jmprel=false reloc_index=3 sym_index=6 provider_object=Some(0) provider_symbol=Some(7)
  resolved_reloc[4] requester=1 is_jmprel=false reloc_index=4 sym_index=11 provider_object=Some(0) provider_symbol=Some(12)
  resolved_reloc[5] requester=1 is_jmprel=false reloc_index=5 sym_index=2 provider_object=Some(0) provider_symbol=Some(4)
  resolved_reloc[6] requester=1 is_jmprel=true reloc_index=0 sym_index=9 provider_object=Some(0) provider_symbol=Some(10)
  resolved_reloc[7] requester=1 is_jmprel=true reloc_index=1 sym_index=4 provider_object=Some(0) provider_symbol=Some(5)
  resolved_reloc[8] requester=2 is_jmprel=false reloc_index=2 sym_index=10 provider_object=Some(0) provider_symbol=Some(11)
  resolved_reloc[9] requester=2 is_jmprel=false reloc_index=3 sym_index=6 provider_object=Some(0) provider_symbol=Some(7)
  resolved_reloc[10] requester=2 is_jmprel=false reloc_index=4 sym_index=12 provider_object=Some(0) provider_symbol=Some(12)
  resolved_reloc[11] requester=2 is_jmprel=false reloc_index=5 sym_index=3 provider_object=Some(0) provider_symbol=Some(4)
  resolved_reloc[12] requester=2 is_jmprel=true reloc_index=0 sym_index=1 provider_object=Some(3) provider_symbol=Some(9)
  resolved_reloc[13] requester=2 is_jmprel=true reloc_index=1 sym_index=9 provider_object=Some(0) provider_symbol=Some(10)
  resolved_reloc[14] requester=2 is_jmprel=true reloc_index=2 sym_index=4 provider_object=Some(0) provider_symbol=Some(5)
  resolved_reloc[15] requester=3 is_jmprel=false reloc_index=2 sym_index=11 provider_object=Some(0) provider_symbol=Some(11)
  resolved_reloc[16] requester=3 is_jmprel=false reloc_index=3 sym_index=6 provider_object=Some(0) provider_symbol=Some(7)
  resolved_reloc[17] requester=3 is_jmprel=false reloc_index=4 sym_index=12 provider_object=Some(0) provider_symbol=Some(12)
  resolved_reloc[18] requester=3 is_jmprel=false reloc_index=5 sym_index=3 provider_object=Some(0) provider_symbol=Some(4)
  resolved_reloc[19] requester=3 is_jmprel=true reloc_index=0 sym_index=10 provider_object=Some(0) provider_symbol=Some(10)
  resolved_reloc[20] requester=3 is_jmprel=true reloc_index=1 sym_index=1 provider_object=Some(2) provider_symbol=Some(11)
  resolved_reloc[21] requester=3 is_jmprel=true reloc_index=2 sym_index=4 provider_object=Some(0) provider_symbol=Some(5)
```
