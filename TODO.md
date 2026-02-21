# VeriLoad TODO

Tracking file for implementation and verification work.

Status keys:
- `[ ]` not started
- `[~]` in progress
- `[x]` complete

## M0: Project Setup

- [ ] Create crate skeleton and module files listed in `design.md`
- [ ] Wire build target (`make veriload` + verify target)
- [ ] Add fail-fast logging helper (single error path)
- [ ] Add trusted runtime module scaffold (`rt.rs`) with explicit contracts

## M1: Spec Foundation (`model.rs`, `spec.rs`, `arith_lemmas.rs`)

- [ ] Define ELF constants and plain structs for Ehdr/Phdr/Dyn/Sym/Rela
- [ ] Define `SegSpec`, `DynInfoSpec`, `ModuleSpec`
- [ ] Define predicates: `mapped_va`, `mapped_range`, `writable_range`, `executable_va`
- [ ] Define `va_to_off` and `off_in_image`
- [ ] Prove arithmetic lemmas for align up/down and checked add/sub
- [ ] Prove `mapped_range -> off_in_image` bridge lemma
- [ ] Prove writable subset lemma (`writable_range -> mapped_range`)

## M2: Verified Parsing (`parse.rs`)

- [ ] Implement checked byte readers (`read_u16_le`, `read_u32_le`, `read_u64_le`)
- [ ] Implement `parse_ehdr`
- [ ] Implement `parse_phdrs`
- [ ] Validate ELF identity/class/endian/machine/type
- [ ] Validate phdr table bounds and entry size
- [ ] Validate `PT_LOAD`/`PT_DYNAMIC` presence
- [ ] Validate `p_vaddr % page == p_offset % page`
- [ ] Reject overflow in `p_offset + p_filesz`, `p_vaddr + p_memsz`
- [ ] Add parser lemmas proving no OOB/overflow in all reads

## M3: Load Plan + Image Materialization (`plan.rs`, `image.rs`)

- [ ] Compute `min_vaddr_page`, `max_vaddr_page`, `image_len`
- [ ] Enforce non-overlapping sorted load segments (reject overlap)
- [ ] Build copy/zero region plan per segment
- [ ] Trusted call: `rt_mmap_rw(image_len)`
- [ ] Implement verified segment copy (`filesz` bytes)
- [ ] Implement verified zero fill (`memsz - filesz`)
- [ ] Prove image bytes match file bytes for copied region
- [ ] Prove zero-fill property for BSS region

## M4: Dynamic Section Parsing (`dynamic.rs`)

- [ ] Implement dynamic vector scan (`DT_*` pairs)
- [ ] Extract and validate `DT_STRTAB`, `DT_STRSZ`, `DT_SYMTAB`, `DT_SYMENT`
- [ ] Extract and validate `DT_RELA`, `DT_RELASZ`, `DT_RELAENT`
- [ ] Extract and validate `DT_JMPREL`, `DT_PLTRELSZ`, `DT_PLTREL`
- [ ] Extract `DT_NEEDED` entries in encounter order
- [ ] Extract `DT_INIT`, `DT_INIT_ARRAY`, `DT_INIT_ARRAYSZ`
- [ ] Reject `DT_REL`, `DT_TEXTREL`, versioning tags
- [ ] Prove all accepted dynamic pointer ranges satisfy `mapped_range`

## M5: Dependency Closure (`deps.rs`)

- [ ] Implement fixed search policy: `/lib`, `/usr/lib`, `/lib64`
- [ ] Implement path resolver (`name -> first existing path`)
- [ ] Implement BFS queue over `DT_NEEDED`
- [ ] Implement dedupe by resolved path string
- [ ] Load modules recursively with same parser/plan/materialize pipeline
- [ ] Prove BFS uniqueness invariant (no duplicate loaded module)
- [ ] Prove deterministic order theorem for module list

## M6: Symbol Resolution (`symbols.rs`)

- [ ] Parse dynsym entries and names safely
- [ ] Implement local symbol resolution (`STB_LOCAL` in module)
- [ ] Implement global strong-only lookup in BFS scope order
- [ ] Ignore weak symbols entirely
- [ ] Reject unresolved symbols
- [ ] Reject out-of-bounds symbol/string table access
- [ ] Prove first-match determinism theorem

## M7: Relocations (`relocate.rs`)

- [ ] Implement RELA iterator with entry-size checks
- [ ] Support `R_X86_64_RELATIVE`
- [ ] Support `R_X86_64_GLOB_DAT`
- [ ] Support `R_X86_64_JUMP_SLOT`
- [ ] Support `R_X86_64_64`
- [ ] Reject `R_X86_64_IRELATIVE` as unsupported in MVP
- [ ] Reject all other relocation kinds
- [ ] Compute relocation place `P = B + r_offset` with checked arithmetic
- [ ] Require `mapped_range(P, 8)` and `writable_range(P, 8)` before writes
- [ ] Implement single write primitive `store_u64_checked`
- [ ] Relocation ordering: dependencies first, main last
- [ ] Proof: each relocation write is in-bounds and policy-compliant

## M8: Final Protections (`protect.rs`)

- [ ] Compute final protection regions from `PF_R/PF_W/PF_X`
- [ ] Page-align protection ranges
- [ ] Apply `mprotect` for segment policies
- [ ] Apply `PT_GNU_RELRO` as read-only after relocations
- [ ] Proof: all `mprotect` ranges are within mapped module image

## M9: Init + Transfer (`init.rs`, `stack.rs`, `main.rs`)

- [ ] Compute dependency-first constructor order (cycle-safe)
- [ ] Call `DT_INIT` then `DT_INIT_ARRAY`
- [ ] Validate init pointers are executable mapped addresses
- [ ] Build minimal stack (`argc=1`, `argv`, `envp=NULL`, no auxv)
- [ ] Trusted jump via `enter(entry, sp)`
- [ ] Ensure `main.rs` orchestrates phases strictly in order

## M10: Negative/Positive Tests

- [ ] Positive: simple PIE with one shared library
- [ ] Positive: multiple DSOs in BFS dependency graph
- [ ] Negative: `DT_REL` -> fatal
- [ ] Negative: `DT_TEXTREL` -> fatal
- [ ] Negative: `R_X86_64_IRELATIVE` -> fatal
- [ ] Negative: TLS presence -> fatal
- [ ] Negative: weak-only unresolved symbol -> fatal
- [ ] Negative: missing `DT_NEEDED` library -> fatal
- [ ] Negative: relocation target outside mapped segment -> fatal

## M11: Proof Quality Gate

- [ ] No `assume(false)` in verified modules
- [ ] No unchecked arithmetic in parser/plan/relocate logic
- [ ] Document every trusted function contract in `rt.rs`
- [ ] Keep trusted code small and isolated
- [ ] Add short proof map in comments (which lemma discharges which safety condition)

## Immediate Next Slice

- [ ] Scaffold modules and stubs
- [ ] Implement `model.rs` + `spec.rs` predicates first
- [ ] Implement parser (`parse_ehdr`, `parse_phdrs`) with proofs
- [ ] Implement load-plan computation and proof of `mapped_range -> off_in_image`
