# VeriLoad TODO

Tracking file for implementation and verification work.

Status keys:
- `[ ]` not started
- `[~]` in progress
- `[x]` complete

## M0: Project Setup

- [x] Create crate skeleton and module files listed in `design.md`
- [x] Wire build target (`make veriload` + verify target)
- [x] Add fail-fast logging helper (single error path)
- [x] Add trusted runtime module scaffold (`rt.rs`) with explicit contracts

## M1: Spec Foundation (`model.rs`, `spec.rs`, `arith_lemmas.rs`)

- [x] Define ELF constants and plain structs for Ehdr/Phdr/Dyn/Sym/Rela
- [x] Define `SegSpec`, `DynInfoSpec`, `ModuleSpec`
- [x] Define predicates: `mapped_va`, `mapped_range`, `writable_range`, `executable_va`
- [x] Define `va_to_off` and `off_in_image`
- [ ] Prove arithmetic lemmas for align up/down and checked add/sub
- [ ] Prove `mapped_range -> off_in_image` bridge lemma
- [ ] Prove writable subset lemma (`writable_range -> mapped_range`)

## M2: Verified Parsing (`parse.rs`)

- [x] Implement checked byte readers (`read_u16_le`, `read_u32_le`, `read_u64_le`)
- [x] Implement `parse_ehdr`
- [x] Implement `parse_phdrs`
- [x] Validate ELF identity/class/endian/machine/type
- [x] Validate phdr table bounds and entry size
- [x] Validate `PT_LOAD`/`PT_DYNAMIC` presence
- [x] Validate `p_vaddr % page == p_offset % page`
- [x] Reject overflow in `p_offset + p_filesz`, `p_vaddr + p_memsz`
- [ ] Add parser lemmas proving no OOB/overflow in all reads

## M3: Load Plan + Image Materialization (`plan.rs`, `image.rs`)

- [x] Compute `min_vaddr_page`, `max_vaddr_page`, `image_len`
- [x] Enforce non-overlapping sorted load segments (reject overlap)
- [x] Build copy/zero region plan per segment
- [x] Trusted call: `rt_mmap_rw(image_len)`
- [x] Implement verified segment copy (`filesz` bytes)
- [x] Implement verified zero fill (`memsz - filesz`)
- [ ] Prove image bytes match file bytes for copied region
- [ ] Prove zero-fill property for BSS region

## M4: Dynamic Section Parsing (`dynamic.rs`)

- [x] Implement dynamic vector scan (`DT_*` pairs)
- [x] Extract and validate `DT_STRTAB`, `DT_STRSZ`, `DT_SYMTAB`, `DT_SYMENT`
- [x] Extract and validate `DT_RELA`, `DT_RELASZ`, `DT_RELAENT`
- [x] Extract and validate `DT_JMPREL`, `DT_PLTRELSZ`, `DT_PLTREL`
- [x] Extract `DT_NEEDED` entries in encounter order
- [x] Extract `DT_INIT`, `DT_INIT_ARRAY`, `DT_INIT_ARRAYSZ`
- [x] Reject `DT_REL`, `DT_TEXTREL`, versioning tags
- [ ] Prove all accepted dynamic pointer ranges satisfy `mapped_range`

## M5: Dependency Closure (`deps.rs`)

- [x] Implement fixed search policy: `/lib`, `/usr/lib`, `/lib64`
- [x] Implement path resolver (`name -> first existing path`)
- [x] Implement BFS queue over `DT_NEEDED`
- [x] Implement dedupe by resolved path string
- [x] Load modules recursively with same parser/plan/materialize pipeline
- [ ] Prove BFS uniqueness invariant (no duplicate loaded module)
- [ ] Prove deterministic order theorem for module list

## M6: Symbol Resolution (`symbols.rs`)

- [x] Parse dynsym entries and names safely
- [x] Implement local symbol resolution (`STB_LOCAL` in module)
- [x] Implement global strong-only lookup in BFS scope order
- [x] Ignore weak symbols entirely
- [x] Reject unresolved symbols
- [x] Reject out-of-bounds symbol/string table access
- [ ] Prove first-match determinism theorem

## M7: Relocations (`relocate.rs`)

- [x] Implement RELA iterator with entry-size checks
- [x] Support `R_X86_64_RELATIVE`
- [x] Support `R_X86_64_GLOB_DAT`
- [x] Support `R_X86_64_JUMP_SLOT`
- [x] Support `R_X86_64_64`
- [x] Reject `R_X86_64_IRELATIVE` as unsupported in MVP
- [x] Reject all other relocation kinds
- [x] Compute relocation place `P = B + r_offset` with checked arithmetic
- [x] Require `mapped_range(P, 8)` and `writable_range(P, 8)` before writes
- [x] Implement single write primitive `store_u64_checked`
- [x] Relocation ordering: dependencies first, main last
- [ ] Proof: each relocation write is in-bounds and policy-compliant

## M8: Final Protections (`protect.rs`)

- [x] Compute final protection regions from `PF_R/PF_W/PF_X`
- [x] Page-align protection ranges
- [x] Apply `mprotect` for segment policies
- [x] Apply `PT_GNU_RELRO` as read-only after relocations
- [ ] Proof: all `mprotect` ranges are within mapped module image

## M9: Init + Transfer (`init.rs`, `stack.rs`, `main.rs`)

- [x] Compute dependency-first constructor order (cycle-safe)
- [x] Call `DT_INIT` then `DT_INIT_ARRAY`
- [x] Validate init pointers are executable mapped addresses
- [x] Build minimal stack (`argc=1`, `argv`, `envp=NULL`, no auxv)
- [x] Trusted jump via `enter(entry, sp)`
- [x] Ensure `main.rs` orchestrates phases strictly in order

## M10: Negative/Positive Tests

- [x] Positive: simple PIE with one shared library
- [x] Positive: multiple DSOs in BFS dependency graph
- [ ] Negative: `DT_REL` -> fatal
- [ ] Negative: `DT_TEXTREL` -> fatal
- [ ] Negative: `R_X86_64_IRELATIVE` -> fatal
- [ ] Negative: TLS presence -> fatal
- [ ] Negative: weak-only unresolved symbol -> fatal
- [x] Negative: missing `DT_NEEDED` library -> fatal
- [ ] Negative: relocation target outside mapped segment -> fatal

## M11: Proof Quality Gate

- [x] No `assume(false)` in verified modules
- [x] No unchecked arithmetic in parser/plan/relocate logic
- [~] Document every trusted function contract in `rt.rs`
- [x] Keep trusted code small and isolated
- [ ] Add short proof map in comments (which lemma discharges which safety condition)

## Immediate Next Slice

- [x] Scaffold modules and stubs
- [x] Implement `model.rs` + `spec.rs` predicates first
- [~] Implement parser (`parse_ehdr`, `parse_phdrs`) with proofs
- [~] Implement load-plan computation and proof of `mapped_range -> off_in_image`
