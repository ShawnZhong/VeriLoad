# VeriLoad Design

## Goal
VeriLoad is a research prototype for an ELF64/x86_64 loader where planning logic is verified in Verus and execution is unverified runtime code.
The planner consumes raw object bytes and produces a full loading plan (`LoaderOutput`).
`tests/main` is the current reference workload.

## End-to-end contract
`plan_loader` (`src/main_impl.rs`) ensures `main_spec::plan_result_spec` (`src/main_spec.rs`).
For `Ok(out)`, `main_spec::plan_ok_spec` requires existence of intermediate stage values that satisfy, in order:
- `parse_stage_spec`
- `discover_stage_spec`
- `resolve_stage_spec`
- `mmap_plan_stage_spec`
- `plan_relocate_stage_spec`
- `relocate_apply_stage_spec`
- `final_stage_spec`

For `Err(_)`, `plan_result_spec` is unconstrained (accepted by spec).

## Scope
Current code path targets:
- ELF64 (`ELFCLASS64`)
- little-endian (`ELFDATA2LSB`)
- executable/shared object inputs (`ET_EXEC`, `ET_DYN`)
- program headers used by planner (`PT_LOAD`, `PT_DYNAMIC`)
- relocations accepted by parser and planner:
  - `R_X86_64_RELATIVE`
  - `R_X86_64_JUMP_SLOT`
  - `R_X86_64_GLOB_DAT`
  - `R_X86_64_COPY`
  - `R_X86_64_64`

The implementation rejects malformed or unsupported inputs with `LoaderError` (fail fast).

## Data model
Input:
- `LoaderInput { objects: Vec<LoaderObject> }`
- `LoaderObject { name: String, bytes: Vec<u8> }`

Key intermediate outputs:
- `DiscoveryResult { order: Vec<usize> }`
- `ResolutionResult { planned: Vec<PlannedObject>, resolved_relocs: Vec<ResolvedReloc> }`
- `RelocatePlanOutput` (mmap plans + reloc plan + carried parsed/discovered/resolved)
- `RelocateApplyOutput` (patched mmap plans + carried metadata)

Final planner output:
- `LoaderOutput` with:
  - `entry_pc`
  - `constructors`
  - `destructors`
  - `mmap_plans`
  - debug/intermediate fields (`reloc_writes`, `parsed`, `discovered`, `resolved`)

## Verified planner stages

### Stage 1: Parse (`parse_impl::parse_stage`)
Spec (`src/parse_spec.rs`):
- one parsed object per input object
- ELF identity and basic format checks (`has_elf_magic`, `has_supported_ident`)
- requires usable dynamic/program-header structure and bounds-safe offsets
- relocation entries must be in supported relocation set

Implementation (`src/parse_impl.rs`) additionally checks concrete ELF header fields (for example `e_machine == EM_X86_64`) before constructing `ParsedObject`.

### Stage 2: Dependency discovery (`discover_impl::discover_stage`)
Spec (`src/discover_spec.rs`):
- discovered order has valid, unique indices
- empty input -> empty order; non-empty input -> first element is object `0`
- direct dependency closure: if an object in order has a dependency edge, target must be in order
- every non-root element has a parent edge from an earlier element

Implementation starts from object `0`, explores dependencies deterministically, deduplicates repeats, and fails if any `DT_NEEDED` in included objects cannot match any provided object SONAME.

### Stage 3: Symbol resolution (`resolve_impl::resolve_stage_ref`)
Spec (`src/resolve_spec.rs`):
- planned scope matches discovered order (`base == 0` placeholders at this stage)
- each recorded `ResolvedReloc` is structurally valid
- provider, if present, must be a symbol-name match with a defined provider symbol
- `None` provider means no matching provider exists in scope

Implementation resolves by scanning objects in discovered order and symbols in symbol-table order, returning the first match found. For required symbol relocations (`JUMP_SLOT`/`GLOB_DAT`/`R_X86_64_64` with non-weak-undefined requester symbol, and all `COPY` relocations), missing provider is an error.

### Stage 4: Mmap planning (`mmap_plan_impl::mmap_plan_stage`)
Spec (`src/mmap_plan_spec.rs`):
- every `MmapPlan` corresponds to some `PT_LOAD` segment in discovered scope
- start addresses/lengths follow page-floor/page-ceil rules
- protections come from ELF `p_flags`
- all mmap ranges are pairwise non-overlapping

Implementation assigns base `0` for `ET_EXEC`, and deterministic dynamic bases for `ET_DYN` using load position.

### Stage 5: Relocation-write planning (`relocate_plan_impl::plan_relocate_stage`)
Spec (`src/relocate_plan_spec.rs`):
- carries forward parsed/discovered/resolved and mmap plans
- `out.reloc_plan` must equal `expected_reloc_writes(...)`
- relocation writes must be sound (`reloc_writes_sound`)

Planned writes include:
- all `RELATIVE` writes from `relas` and `jmprels`
- symbol-based writes for resolved `JUMP_SLOT`, `GLOB_DAT`, `R_X86_64_64`, and `COPY`

### Stage 6: Relocation-write apply (`relocate_apply_impl::relocate_apply_stage`)
Spec (`src/relocate_apply_spec.rs`):
- output mmap bytes equal applying all relocation writes to input mmap bytes
- layout is preserved (`same_mmap_layout`)
- parsed/discovered/resolved and relocation records are preserved
- mmap and relocation soundness properties are retained

Writes are modeled as little-endian 8-byte patches at computed write addresses.

### Stage 7: Final output assembly (`final_stage_impl::final_stage`)
Spec (`src/final_stage_spec.rs`):
- `entry_pc` equals expected entry address from parsed entry + computed object base
- constructors and destructors are sound (each PC comes from a valid init/fini array entry)
- mmap/relocation/planner metadata is preserved and remains sound

Implementation ordering:
- constructors: reverse discovered object order, each object's `init_array` in forward order
- destructors: forward discovered object order, each object's `fini_array` in reverse order

## Unverified boundaries

### Stage 0: Input setup (`read_loader_input`)
- reads each CLI-provided path into bytes
- derives object name from filename
- builds `LoaderInput`

No Verus spec is attached to this stage.

### Stage 8: Runtime execution (`runtime::run_runtime`)
Runtime executes `LoaderOutput` by:
1. mapping each planned region
2. copying planned bytes
3. applying final memory protections
4. calling constructors
5. transferring control to `entry_pc` with a minimal stack
6. calling destructors after entry returns

Relocation writes are already reflected in planned bytes before runtime; runtime does not perform a separate relocation pass.

## Build and check
- Verify planner proofs: `make verify`
- Build loader and test artifacts: `make`
- Run sample workload: `./run.sh`
- Run with plan debug dump: `./run.sh --debug`
