# VeriLoad Detailed Design (Verification-First)

This document is the implementation and proof plan for a minimal x86-64 ELF dynamic loader in Verus.
It is intentionally strict, fail-fast, and scoped for research-prototype verification.

## 0) Fixed Policies

These are fixed unless explicitly changed:

- Architecture/ABI: ELF64, little-endian, `EM_X86_64`
- Object type: main executable `ET_DYN`, DSOs `ET_DYN`
- Search path: exactly `/lib`, `/usr/lib`, `/lib64`
- No auxv at program entry
- Process entry setup: `argc = 1`, `argv = [path, NULL]`, `envp = NULL`
- Relocation formats: RELA only (`DT_REL` rejected)
- Relocations supported: `R_X86_64_RELATIVE`, `R_X86_64_GLOB_DAT`, `R_X86_64_JUMP_SLOT`, `R_X86_64_64`
- Eager binding only (`DT_JMPREL` always processed before jump)
- Weak symbols ignored during lookup
- Reject `R_X86_64_COPY`, `R_X86_64_IRELATIVE`, TLS (`PT_TLS` and TLS relocations), text relocations (`DT_TEXTREL`), symbol versioning (`DT_VERSYM`/`DT_VERNEED`/`DT_VERDEF`)
- Program headers: require `PT_LOAD` and `PT_DYNAMIC`; allow `PT_GNU_RELRO`, `PT_PHDR`, `PT_INTERP` (ignored for loading decisions)

Any unsupported feature or invariant violation is fatal: log + panic.

## 1) What We Took From References

### 1.1 musl/ldso reference points

The following behaviors guide this design (simplified for our scope):

- Address translation model (`laddr`): VA translated by one base offset for non-FDPIC targets (`musl/ldso/dynlink.c:215`).
- Dynamic-vector decode model (`decode_vec`, `search_vec`): linear scan over `DT_*` pairs (`musl/ldso/dynlink.c:220`, `musl/ldso/dynlink.c:231`).
- Mapping model: compute global min/max load range, map, copy, zero-fill BSS, enforce `PT_DYNAMIC` presence (`musl/ldso/dynlink.c:687`).
- Dependency construction: direct `DT_NEEDED` first, then BFS closure (`musl/ldso/dynlink.c:1261`, `musl/ldso/dynlink.c:1305`).
- Relocation flow: resolve symbol, apply reloc equation, fail loudly on unresolved/unsupported (`musl/ldso/dynlink.c:381`).
- Post-relocation RELRO protection (`musl/ldso/dynlink.c:1425`).

### 1.2 Verus examples reference points

Verification style and structure are based on:

- Function contracts via `requires`/`ensures` (`verus/examples/guide/requires_ensures.rs:13`).
- Loop invariants and decreases clauses for executable loops (`verus/examples/guide/invariants.rs:112`).
- Explicit overflow-prevention preconditions for machine integers (`verus/examples/guide/integers.rs:85`).
- Trigger-aware quantified invariants (`verus/examples/guide/quants.rs:81`).
- Ghost/tracked state pattern for memory/resource invariants (`verus/examples/verified_vec.rs:24`).

## 2) TCB and Trust Boundaries

Trusted components:

1. Linux kernel syscalls (`open`, `pread`, `mmap`, `mprotect`, `close`)
2. Entry trampoline (`enter(entry, stack_ptr) -> !`)
3. Runtime wrappers converting raw mappings into slices

Trusted contracts required:

- `rt_mmap_rw(len)` returns fresh writable zeroed memory or fails.
- `rt_mprotect(addr, len, prot)` changes permissions only.
- `rt_pread_exact(fd, off, buf)` fills exactly `buf.len()` or fails.
- `enter` never returns.

All non-TCB logic is verified and must not use unchecked pointer arithmetic.

## 3) Formal Model

### 3.1 Spec data types

- `SegSpec`: one `PT_LOAD` segment
  - `vaddr: u64`
  - `memsz: u64`
  - `filesz: u64`
  - `fileoff: u64`
  - `flags: u32` (`PF_R|PF_W|PF_X`)
- `DynInfoSpec`: validated dynamic-table info
  - dynstr range, dynsym range
  - rela/jmprel ranges and entry sizes
  - needed-name offsets
  - init/init_array pointers
- `ModuleSpec`:
  - `base: u64`
  - `min_vaddr_page: u64`
  - `max_vaddr_page: u64`
  - `image_len: usize`
  - `segs: Seq<SegSpec>`
  - `dyn: DynInfoSpec`

### 3.2 Core predicates

Use these in all proofs:

- `seg_contains_va(seg, va)`
- `seg_contains_range(seg, va, n)`
- `mapped_va(m, va) := exists seg. seg_contains_va(seg, va)`
- `mapped_range(m, va, n) := exists seg. seg_contains_range(seg, va, n)`
- `writable_range(m, va, n) := exists seg. PF_W && seg_contains_range(seg, va, n)`
- `executable_va(m, va) := exists seg. PF_X && seg_contains_va(seg, va)`
- `va_to_off(m, va) = va - m.min_vaddr_page`
- `off_in_image(m, off, n) := off + n <= m.image_len`

### 3.3 Strong VA safety rule

No read/write is allowed from interval-only reasoning.

Required rule for every access:

- If reading/writing `n` bytes at virtual address `va`, prove `mapped_range(m, va, n)`.
- Then prove `off_in_image(m, va_to_off(m, va), n)` before touching `image`.

This blocks false positives caused by holes between `PT_LOAD` segments.

## 4) Global Invariants

For each loaded module:

1. Segment shape:
- `filesz <= memsz`
- `fileoff + filesz <= file_len`
- `vaddr + memsz` has no overflow
- `p_vaddr mod page_size == p_offset mod page_size`

2. Segment order policy (simplification for proof):
- Load segments are sorted by `vaddr`
- No overlapping `mem` ranges
- Any overlap in input is rejected

3. Image layout:
- `image_len == align_up(max(vaddr+memsz) - min_vaddr_page, page_size)`
- Loaded file bytes match exactly
- `memsz-filesz` region is zero

4. Dynamic pointers:
- Every `DT_*` pointer used by MVP maps to `mapped_range` of required size
- `DT_SYMENT == sizeof(Elf64_Sym)`, `DT_RELAENT == sizeof(Elf64_Rela)`

5. Relocation safety:
- Each relocation write target satisfies `mapped_range(..., 8)` and `writable_range(..., 8)`.

## 5) Deterministic Algorithms (Spec)

### 5.1 Dependency search and load order

Given SONAME `name`, candidate paths are:

1. `/lib/name`
2. `/usr/lib/name`
3. `/lib64/name`

First readable path wins; if none exists, fail.

Dependency traversal is BFS:

- Start with main module `DT_NEEDED` in declared order.
- Pop queue head, load if not already loaded, append its direct `DT_NEEDED` in order.
- Dedup key: resolved absolute path string.

Proof goal: resulting module list and global symbol scope are deterministic.

### 5.2 Symbol resolution

Input: requesting relocation `(module M, symbol index i)`.

- If binding is local (`STB_LOCAL`), resolve in `M` only.
- Else scan global scope in BFS order.
- Eligible definition must satisfy:
  - defined (`st_shndx != SHN_UNDEF`)
  - non-weak (`STB_GLOBAL` only; weak ignored)
- First eligible definition wins.
- If none, fail.

No version filtering in MVP.
No GNU hash requirement in MVP (linear dynsym scan only).

## 6) Relocation Semantics

Notation:

- `B`: module base
- `A`: addend (`r_addend`)
- `S`: resolved symbol address
- `P`: relocation place address (`B + r_offset`)

Supported equations:

- `R_X86_64_RELATIVE`: `*P = B + A`
- `R_X86_64_GLOB_DAT`: `*P = S`
- `R_X86_64_JUMP_SLOT`: `*P = S`
- `R_X86_64_64`: `*P = S + A`

Relocation procedure for each entry:

1. Decode entry and relocation kind.
2. Compute `P` with checked arithmetic.
3. Prove `mapped_range(M, P, 8)` and `writable_range(M, P, 8)`.
4. Resolve symbol (if required by kind).
5. Compute value with checked arithmetic.
6. Store via single primitive `store_u64_checked(image, off, val)`.

All unsupported relocation types are fatal.

## 7) End-to-End Loader Phases

### Phase A: Parse ELF

- Parse Ehdr and Phdrs with bounds checks.
- Require at least one `PT_LOAD` and one `PT_DYNAMIC`.
- Reject non-target format immediately.

Proof obligations:

- No out-of-bounds reads.
- No integer-overflow in offset/length arithmetic.

### Phase B: Build load plan

- Collect and validate all `PT_LOAD` segments.
- Compute `min_vaddr_page`, `max_vaddr_page`, `image_len`.
- Record copy/zero regions and final protections.

Proof obligations:

- Plan spans all segment memory and nothing outside span.

### Phase C: Map and materialize image

- `mmap` contiguous RW image.
- Copy `filesz` bytes for each segment.
- Zero-fill `memsz-filesz` tail.

Proof obligations:

- Every copy/zero operation is in-bounds and consistent with plan.

### Phase D: Parse dynamic section

- Decode `DT_*` entries from mapped memory.
- Validate dynstr/dynsym/rela/jmprel/init ranges.

Proof obligations:

- Every derived pointer range is `mapped_range`.

### Phase E: Build dependency closure (BFS)

- Resolve/load each `DT_NEEDED` in queue order.
- Deduplicate and append next-level dependencies.

Proof obligations:

- Queue invariants: no duplicate load, deterministic output order.

### Phase F: Relocation

Per module order:

1. non-symbol pass (`RELATIVE`)
2. symbol pass (`GLOB_DAT`, `JUMP_SLOT`, `64`)

Module order policy:

- Relocate dependencies before dependents.
- Main module relocated last.

Proof obligations:

- All writes are mapped + writable + 8-byte aligned handling policy.
- Symbol index and string accesses are in bounds.

### Phase G: Final permissions and RELRO

- Apply segment protections from `p_flags`.
- If `PT_GNU_RELRO` exists, set RO after relocations.

Proof obligations:

- `mprotect` ranges are page-aligned and within mapped image.

### Phase H: Initializers and transfer

- Compute constructor order: dependency-first traversal (cycle-safe visited set).
- For each module in order: call `DT_INIT` then `DT_INIT_ARRAY` entries.
- Build minimal stack (no auxv) and jump to entry.
- Entry address rule: for `ET_DYN`, runtime entry address is `base + e_entry`.

Proof obligations:

- Every init function pointer is from validated mapped executable range.
- `enter` is terminal.

## 8) Verus Module Design

Planned project tree:

- `model.rs`: ELF structs/constants and plain runtime structs.
- `spec.rs`: spec predicates, invariants, and phase-state model.
- `arith_lemmas.rs`: alignment/overflow lemmas.
- `parse.rs`: verified byte decoding and header parsing.
- `plan.rs`: verified load-plan computation.
- `image.rs`: verified copy/zero against plan.
- `dynamic.rs`: verified dynamic-table parsing/validation.
- `deps.rs`: verified BFS closure and deterministic order.
- `symbols.rs`: verified symbol lookup over scope order.
- `relocate.rs`: verified relocation application.
- `protect.rs`: verified protection-range computation.
- `init.rs`: verified constructor-order computation.
- `stack.rs`: minimal stack image builder.
- `rt.rs`: trusted syscall/entry wrappers.
- `main.rs`: orchestration and fail-fast logging.

## 9) Proof Roadmap and Key Lemmas

### 9.1 Arithmetic lemmas

- `lemma_align_down_le(x, p)`
- `lemma_align_up_ge(x, p)`
- `lemma_align_up_sub_no_overflow(max, min, p)`
- `lemma_checked_add_u64(a, b)`

### 9.2 Mapping lemmas

- `lemma_mapped_range_implies_mapped_va`
- `lemma_mapped_range_to_off_in_image`
- `lemma_writable_range_implies_mapped_range`

### 9.3 Parser lemmas

- `lemma_read_u16_bounds`
- `lemma_read_u32_bounds`
- `lemma_read_u64_bounds`
- `lemma_parse_phdr_list_sound`

### 9.4 Dependency/order lemmas

- `lemma_bfs_queue_unique`
- `lemma_bfs_output_deterministic`
- `lemma_ctor_order_dep_before_user`

### 9.5 Relocation lemmas

- `lemma_rela_entry_bounds`
- `lemma_symbol_index_bounds`
- `lemma_store_u64_safe`
- `lemma_reloc_write_policy`

## 10) Testing and Validation Plan

The runtime test target is Alpine Linux in a container.
All tests are executed there, not on the host.

### 10.1 Alpine test environment

- Start from `alpine:latest`.
- Install minimal build/debug tools (`build-base`, `binutils`, `musl-dev`, `bash`, `file`).
- Build loader and fixtures inside the container.
- Keep test DSOs only in supported search paths: `/lib`, `/usr/lib`, `/lib64`.
- Every test run starts from a clean container state.

### 10.2 Test harness contract

Define one test harness script that runs all cases and fails on first unexpected result.

Each test case declares:
- input executable path
- DSO layout (files copied into `/lib`, `/usr/lib`, `/lib64`)
- expected result (`success` or `fatal`)
- expected log token(s)

Pass/fail rules:
- `success`: loader exits 0 and reaches transfer to entry.
- `fatal`: loader exits non-zero and prints a clear fatal reason.
- Any mismatch is immediate failure.

### 10.3 Positive runtime cases

- `pos_minimal_pie`: PIE with `R_X86_64_RELATIVE` only.
- `pos_one_dso`: main + one DSO using `GLOB_DAT`/`JUMP_SLOT`.
- `pos_multi_dso_bfs`: dependency graph that confirms BFS load order.
- `pos_abs64`: case that exercises `R_X86_64_64`.

For positives, also assert deterministic behavior by running each case multiple times and checking identical load order logs.

### 10.4 Negative runtime cases (must fail loudly)

- `neg_dt_rel`: binary contains `DT_REL`.
- `neg_dt_textrel`: binary contains `DT_TEXTREL`.
- `neg_irelative`: relocation contains `R_X86_64_IRELATIVE`.
- `neg_tls`: `PT_TLS` or TLS relocation is present.
- `neg_missing_symbol`: unresolved symbol (including weak-only availability).
- `neg_missing_needed`: `DT_NEEDED` cannot be resolved in fixed search paths.
- `neg_bad_reloc_target`: relocation target not mapped/writable.

Each negative test verifies both non-zero exit and a specific fatal log string.

### 10.5 Proof and trust gates

- No `assume(false)` in verified modules.
- All key lemmas in Section 9 are proved.
- Trusted module (`rt.rs`) stays small and only contains syscall/entry wrappers.
- Trusted contracts are documented at call sites and in `rt.rs`.

## 11) Known Simplifications and Non-Goals

- No glibc compatibility work.
- No lazy binding.
- No runtime `dlopen` semantics in MVP.
- No symbol versioning.
- No TLS support.
- No compatibility fallbacks.

This is deliberate for proof tractability and clear failure modes.
