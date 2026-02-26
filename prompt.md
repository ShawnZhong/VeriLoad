# Objective
Based on ELF documentation and the x86-64 psABI, derive and implement a formal Verus specification and implementation for an ELF dynamic loader in `src/`.

# Workflow
- Read standards first, then for each stage write spec, implementation, and refinement proofs.
  - `src/<stage>_spec.rs` defines the specification with data models and constants based on the gABI and psABI.
  - `src/<stage>_impl.rs` implements the specification with refinement proofs.
  - `src/main_spec.rs` and `src/main_impl.rs` orchestrate the stages and calls into the runtime.
  - `src/runtime.rs` executes the final plan.
  - `src/debug.rs` should print intermediate planner results and final output for inspection.
- After each stage, run proofs and test on sample input (`./run.sh --debug`).
- For debugging, use `readelf`, `objdump`, and `ldd` to inspect ELF layout, disassembly/relocations, and runtime shared-library dependencies.

# Best Practices
- Define a small mathematical spec fn first, then prove/implement against it (spec -> lemmas -> exec).
- Make sure the spec is easy for a human to read and understand with citation to the standard.
- Keep contracts minimal and semantic: requires only for real assumptions, ensures for behavior.
- Fallible stage APIs should use `Option<T>`. Use `Vec<u8>` for bytes-related data.
- Keep the trusted boundary small: minimize `external_body`/`assume_specification`.
- Keep code simple and explicit (research prototype).

# Stages
- Unverified input setup: read executable/shared objects from filesystem and build planner input as a sequence of ELF objects (`name + raw bytes`), where `name` is derived from `Path::file_name()`.
- Verified planner: `byte decode -> semantic validation/normalization -> dependency graph construction -> dependency ordering -> symbol resolution -> memory mapping planning -> relocation write planning -> init/fini planning and finalization`; output includes entry address, ordered constructors/destructors, memory mapping plans (`object_name`, `start`, `bytes`, `protection`), relocation writes (`address`, `bytes`, `type`).
- Unverified runtime: execute final plan in explicit order (`map -> apply planned relocation writes -> protect -> build initial stack/auxv/argv -> call constructors -> jump entry`).

## 0. Program Invocation (unverified)
- Input: `{ CLI input is `veriload [--debug] <elf> [<elf> ...] [-- <args> ...]`, and referenced files are available on disk }`
- Output: `{ Initialization builds planner input as an ordered object list from the `<elf>` arguments, where each object is (`name: Vec<u8>`, `bytes: Vec<u8>`), `name` is derived from the input path basename, `bytes` is the full file content, and object index 0 is the first `<elf>` argument (the root object for dependency stages). The planner object universe is exactly this list (no implicit filesystem/library-path search for `DT_NEEDED`). It then invokes the verified planning pipeline, optionally prints debug plan data, and hands final plan plus forwarded `<args>` to runtime (`<args>` is empty if `--` is absent). }`

## 1. Byte Decode
- Input: `{ Object set represented as raw bytes }`
- Output: `{ Raw decoded ELF structures (headers, tables, offsets, relocation records, dynamic entries) with structural decoding and bounds safety }`
- Spec: gABI `02-eheader.rst` (`ELF Header`, `Contents of the ELF Header`, `ELF Identification`, `Data Encoding`); `07-pheader.rst` (`Program Header Entry`, `Segment Types`); `08-dynamic.rst` (`Dynamic Section`); `06-reloc.rst` (`Relocation Entry`, `Relative Relocation Table`); psABI `object-files.tex` (`ELF Header`, `Machine Information`, `Number of Program Headers`).
- Include decoding for dynamic relocation sources used by loaders (`RELA`/PLT relocation tables and RELR-formatted data where present).

## 2. Semantic Validation and Normalization
- Input: `{ Raw decoded ELF structures }`
- Output: `{ Canonical parsed objects suitable for later stages, with semantic ELF/ABI checks, normalized string/name views, and constants converted into validated modeled forms }`
- Spec: gABI `02-eheader.rst` (`Contents of the ELF Header`, `ELF Identification`); `04-strtab.rst` (`String Table`); `05-symtab.rst` (`Symbol Table`, `Symbol Table Entry`, `Symbol Binding`, `Symbol Type`, `Symbol Visibility`, `Section Index`); `06-reloc.rst` (`Relocation Entry`); `08-dynamic.rst` (`Dynamic Section`); psABI `object-files.tex` (`ELF Header`, `Symbol Table`, `Relocation`).
- Enforce internal consistency for metadata required by dependency, symbol, relocation, and init/fini stages.

## 3. Dependency Graph Construction
- Input: `{ Parsed objects with `DT_NEEDED` and SONAME/name data (from Stage 0 object list only) }`
- Output: `{ A dependency graph rooted at object index 0, built with BFS expansion and stable edge matching }`
- Spec: gABI `08-dynamic.rst` (`Dynamic Section`, `Shared Object Dependencies`) for `DT_NEEDED` and `DT_SONAME`.
- Dependency matching rule: each `DT_NEEDED` entry matches provider `DT_SONAME` when present; otherwise it matches provider `input_name` (basename-derived loader object name).
- Graph construction must be deduplicated, and cycle-safe.
- If a `DT_NEEDED` entry has no matching provider in the Stage 0 object set, return error.

## 4. Dependency Ordering
- Input: `{ Dependency graph rooted at object index 0 }`
- Output: `{ A valid dependency order for loading/resolution, with uniqueness and closure properties }`
- Spec: gABI `08-dynamic.rst` (`Dynamic Section`, `Shared Object Dependencies`) for `DT_NEEDED` and `DT_SONAME`.

## 5. Symbol Resolution
- Input: `{ Parsed objects + dependency order }`
- Output: `{ Every relevant relocation has a provider decision consistent with symbol visibility/definition rules and load scope }`
- Spec: gABI `06-reloc.rst` (`Relocation Entry`); `08-dynamic.rst` (`Dynamic Section`, `Hash Table`); psABI `object-files.tex` (`Symbol Table`, `Relocation`, `Relocation Types`) and `dl.tex` (`Dynamic Section`).
- Provider search order and strong/weak precedence follow psABI `dl.tex` lookup semantics over the Stage 4 dependency order.
- For required relocations, unresolved providers are errors except for ABI-allowed weak-undefined behavior.

## 6. Memory Mapping
- Input: `{ Parsed objects + dependency order }`
- Output: `{ A non-overlapping sequence of mmap plans with correct addresses, bytes, and protection flags from load segments }`
- Spec: gABI `07-pheader.rst` (`Program Header Entry`, `Segment Types`, `Base Address`, `Segment Permissions`, `Segment Contents`) and `02-eheader.rst` (`Contents of the ELF Header`).
- Use page-floor/page-ceil segment mapping rules; `ET_EXEC` objects map with base 0, `ET_DYN` objects use deterministic page-aligned non-overlapping bases in dependency order, starting from a fixed implementation constant.
- Segment image bytes must follow load semantics: file-backed bytes plus zero-initialized region where required.

## 7. Relocation
- Input: `{ Parsed objects + dependency order + resolved symbols + mmap plans }`
- Output: `{ A relocation write plan consistent with relocation semantics and symbol/provider decisions}`
- Spec: gABI `06-reloc.rst` (`Relocation Entry`, `Relative Relocation Table`); `08-dynamic.rst` (`Dynamic Section`); psABI `object-files.tex` (`Relocation`, `Relocation Types`) and `dl.tex` (`Dynamic Section`).
- Must implement at least:
  - `R_X86_64_RELATIVE`
  - `R_X86_64_JUMP_SLOT`
  - `R_X86_64_GLOB_DAT`
  - `R_X86_64_COPY`
  - `R_X86_64_64`
  - `R_X86_64_DTPMOD64`
  - `R_X86_64_DTPOFF64`
  - `R_X86_64_TPOFF64`
- Other relocation table forms and addend/base semantics follow gABI/psABI; unsupported forms must be rejected explicitly.

## 8. Init/Fini
- Input: `{ Memory mapping plan + relocation write plan + parsed/dependency/resolved metadata }`
- Output: `{ Final loader plan contains entry address plus ordered constructor/destructor call plans, along with memory mappings, relocation writes}`
- Spec: gABI `02-eheader.rst` (`Contents of the ELF Header`, entry field), `08-dynamic.rst` (`Dynamic Section`, `Initialization and Termination Functions` for `INIT_ARRAY`/`FINI_ARRAY`); psABI `dl.tex` (`Initialization and Termination Functions`).
- Entry address is computed from program entry plus chosen base.
- Constructor/destructor ordering follows gABI/psABI initialization and termination rules over the Stage 4 dependency order.
- `DT_PREINIT_ARRAY` is out of scope for this prototype.

## 9. Runtime Execution (unverified)
- Input: `{ Final loader plan (`entry_pc`, constructors, destructors, mmap plans, relocation write plan) + CLI `<args>` forwarded from Program Invocation }`
- Output: `{ Runtime maps planned memory at fixed addresses, applies planned relocation writes directly to mapped memory, applies final page protections, builds an initial stack/auxv/argv image with forwarded `<args>`, runs constructors, and transfers control to `entry_pc` }`
- Spec: psABI `low-level-sys-info.tex` (`Process Initialization`, `Initial Stack and Register State`, `Stack State`, `Auxiliary Vector`, including `AT_*` entries such as `AT_PHDR`, `AT_PHENT`, `AT_PHNUM`, `AT_ENTRY`, `AT_RANDOM`, `AT_EXECFN`); gABI `02-eheader.rst` (`Contents of the ELF Header`), `07-pheader.rst` (`Program Header Entry`, `Segment Types`), and `08-dynamic.rst` (`Initialization and Termination Functions`).
- Implementation boundary: unverified runtime in `src/runtime.rs` using host `mmap`/`mprotect` and auxv/uid/gid queries.
- Stack/auxv/argv contract: runtime builds `argc = 1 + len(<args>)`, `argv[0]=program-name` (from main object `input_name`), `argv[1..]=forwarded <args>`, empty `envp`, and auxv ending with `AT_NULL`. Auxv layout and required entries follow psABI `low-level-sys-info.tex`; include at least `AT_PAGESZ`, `AT_BASE`, `AT_ENTRY`, `AT_RANDOM`, `AT_EXECFN`, and `AT_PHDR`/`AT_PHENT`/`AT_PHNUM` when available.
- Unofficial VeriLoad-musl contract: each constructor is invoked as `extern "C" fn(*mut *mut i8, *mut i8)` and receives `(envp, program_name_ptr)`.

# Acceptance Checks
1. `make verify`
2. `make`
3. `./run.sh` (and `./run.sh --debug` for plan inspection)
4. `./run_alpine.sh`


# Reading List
## gABI (`third_party/gABI/docsrc/elf`)
- `02-eheader.rst` - ELF identity, class, endianness, machine, entry metadata.
- `03-sheader.rst` - section model background (useful context, even if loader is segment-driven).
- `04-strtab.rst` - string table encoding used by names and dependencies.
- `05-symtab.rst` - symbol table structure and symbol fields.
- `06-reloc.rst` - relocation record format, addends, relocation computation model.
- `07-pheader.rst` - program headers, `PT_LOAD`, `PT_DYNAMIC`, memory protection flags.
- `08-dynamic.rst` - dynamic tags (`DT_NEEDED`, `DT_SONAME`, relocation/init/fini tags).

## psABI (`third_party/psABI_x86_64/x86-64-ABI`)
- `object-files.tex` - x86-64 object and relocation semantics required for correctness.
- `dl.tex` - dynamic linking conventions and symbol lookup context.
- `low-level-sys-info.tex` - process initialization details, initial stack layout, and auxiliary vector (`auxv`/`AT_*`) semantics.

## Verus References (`third_party/verus`)
- `examples/guide/requires_ensures.rs` - clean stage-level contract style.
- `examples/guide/exec_spec.rs` - mapping executable code to spec functions.
- `examples/guide/invariants.rs` - loop invariants for parser/discovery passes.
- `examples/guide/recursion.rs` - recursive spec helpers and decreases patterns.
- `examples/guide/quants.rs` - quantified properties (`forall`/`exists`) used in stage specs.
- `examples/guide/calc.rs` - arithmetic/algebraic proof chains.
- `examples/guide/integers.rs` - integer reasoning patterns for offsets, bounds, and addresses.
- `source/docs/guide/src/triangle.md` - compact end-to-end refinement workflow.
- `source/docs/guide/src/interacting-with-unverified-code.md` - verified/unverified boundary patterns.
- `source/docs/guide/src/trigger-annotations.md` - trigger control when quantified proofs become unstable.
