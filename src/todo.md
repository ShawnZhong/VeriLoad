# VeriLoad TODO

## Stage 0: Setup
- [x] Read `docs/elf.md`
- [x] Read `docs/x86_64.md` as needed
- [x] Review Verus examples (`triangle.md`, `examples/guide/recursion.rs`)
- [x] Write `src/design.md`
- [x] Write `src/todo.md`

## Stage 1: Parse
- [x] Define parse-stage spec relation and parse data structs
- [x] Implement ELF64 little-endian readers with bounds checks
- [x] Parse ELF header and program headers
- [x] Parse dynamic table fields needed by later stages
- [x] Parse dynstr/dynsym/rela/jmprel/init_array data
- [x] Prove parse implementation refines parse spec

## Stage 2: Dependency discovery
- [x] Define dependency-stage spec relation
- [x] Implement closure and deterministic load order
- [x] Implement initializer object ordering
- [x] Prove dependency implementation refines spec

## Stage 3: Symbol resolution
- [x] Define symbol-resolution stage spec relation
- [x] Implement cross-object symbol lookup in load scope
- [x] Prove resolution implementation refines spec

## Stage 4: Relocation + mapping plan
- [x] Define relocation/mmap stage spec relation
- [x] Implement PT_LOAD to `MmapPlan` conversion
- [x] Implement relocation write computation (`RELATIVE`, `JUMP_SLOT`)
- [x] Implement byte patching in mmap plans
- [x] Compute entry PC and ordered initializer PCs
- [x] Prove relocation/mapping implementation refines spec

## End-to-end
- [x] Compose stage pipeline
- [x] Add end-to-end refinement theorem
- [x] Ensure `LoaderOutput` contains required fields

## Verification boundary implementation
- [ ] Init code: read executable + needed libs and build `LoaderInput`
- [ ] Runtime code: mmap, protect, apply writes, call ctors, setup stack, jump entry
- [ ] Add x86_64 jump assembly stub

## Testing
- [ ] Add planner tests for dependency order
- [ ] Add planner tests for symbol resolution + relocations
- [ ] Add planner tests for mmap plans and initializer order
- [x] Run `make verify`
- [x] Run `make build`
- [x] Run `./veriload --debug tests/build/main tests/build/libfoo.so tests/build/libbar.so tests/build/libbaz.so`
- [x] Run `./veriload tests/build/main tests/build/libfoo.so tests/build/libbar.so tests/build/libbaz.so`
