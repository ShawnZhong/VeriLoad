# VeriLoad Design

## Goal
Build a research prototype ELF dynamic loader for ELF64/x86_64 that:
- accepts `LoaderInput` as object name + raw bytes
- computes verified loader planning output
- executes the plan at the unverified runtime boundary
- can load and run `tests/main`

The verified model is stage-based and refines stage specs to implementation outputs.

## Scope
Supported object model:
- ELF class: `ELFCLASS64`
- Endianness: `ELFDATA2LSB`
- Machine: `EM_X86_64`
- File types: `ET_EXEC`, `ET_DYN`
- Program headers: `PT_LOAD`, `PT_DYNAMIC`
- Dynamic tags: `DT_NEEDED`, `DT_STRTAB`, `DT_STRSZ`, `DT_SYMTAB`, `DT_SYMENT`, `DT_RELA`, `DT_RELASZ`, `DT_RELAENT`, `DT_JMPREL`, `DT_PLTRELSZ`, `DT_PLTREL`, `DT_INIT_ARRAY`, `DT_INIT_ARRAYSZ`, `DT_NULL`
- Relocations: `R_X86_64_RELATIVE`, `R_X86_64_JUMP_SLOT`

Fail-fast behavior:
- reject unsupported ELF classes/endianness/machine/reloc types
- reject malformed offsets/sizes
- panic on fatal loader/runtime errors

## Data Model

### Input
- `LoaderInput { objects: Vec<LoaderObject> }`
- `LoaderObject { name: String, bytes: Vec<u8> }`

### Verified-stage internal model
- `ParsedObject`:
  - ELF header fields (type, entry)
  - PT_LOAD list
  - PT_DYNAMIC location
  - dynamic metadata (needed names, strtab/symtab/rela/jmprel/init_array pointers)
  - dynsym + dynstr
  - relocations

### Output
- `LoaderOutput`:
  - `entry_pc: u64`
  - `constructors: Vec<InitCall>` (ordered)
  - `destructors: Vec<TermCall>` (ordered)
  - `mmap_plans: Vec<MmapPlan>`
  - `reloc_writes: Vec<RelocWrite>`

`MmapPlan` contains:
- starting address
- bytes to map
- protection flags

## Staged pipeline

### Stage 1: Parse
Spec:
- each input object bytes parse to one well-formed `ParsedObject`
- constants and layout checks enforced
- required dynamic tables extracted from bytes

Implementation:
- little-endian readers + bounds checks
- ELF/program header parse
- dynamic section parse
- dynstr/dynsym parse
- RELA/JMPREL parse

Refinement:
- parser output satisfies `parse_stage_spec(input, parsed)`

### Stage 2: Dependency discovery
Spec:
- discover transitive closure from main object via `DT_NEEDED`
- produce deterministic load order (BFS)
- produce initializer object order (reverse load order)

Implementation:
- map needed soname -> object index
- queue-based traversal
- deduplicate by first visit

Refinement:
- `discover_stage_refines(parsed, discovered)`

### Stage 3: Symbol resolution
Spec:
- resolve relocation symbol references against load scope
- global scope is load order
- first definition wins

Implementation:
- symbol table scan over objects in load order
- record resolved addresses and relocation writes

Refinement:
- `resolve_stage_refines(parsed, discovered, resolved)`

### Stage 4: Relocation + mapping plan
Spec:
- generate mmap plans from PT_LOAD segments
- determine base addresses (`ET_EXEC` fixed, `ET_DYN` assigned deterministic base windows)
- evaluate relocation writes (`RELATIVE`, `JUMP_SLOT`)
- compute entry PC and initializer PCs

Implementation:
- build segment byte images (`p_filesz` bytes + zero-fill to `p_memsz`)
- compute absolute addresses
- patch relocation values into plan bytes
- emit ordered initializer list

Refinement:
- `reloc_stage_refines(...)`

### End-to-end
Spec:
- loader output is the result of stage composition

Implementation:
- `plan_loader(input)` calls parse -> discover -> resolve -> relocate

Refinement:
- `loader_end_to_end_refines(input, output)`

## Verification boundary

### Unverified initialization code
Responsibilities:
- read main ELF file and all shared libraries (.so files) from same directory from the filesystem.
- build `LoaderInput`
- call verified planner

### Unverified runtime code
Responsibilities:
- `mmap` each `MmapPlan`
- copy bytes, set protections
- apply relocation writes
- call constructors in order
- build initial stack
- jump to entry with small x86_64 assembly stub

## Testing strategy
- Verus verification:
  - run `make verify` after changes
- Loader planner checks:
  - dependency order for test graph
  - symbol resolution + relocation values
  - mmap plan addresses/bytes/protections
  - initializer order
- Runtime execution:
  - run `veriload tests/build/main`
  - expect successful execution and `PASS`
