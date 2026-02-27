# Objective
Based on ELF documentation and the x86-64 psABI, derive and implement a formal Verus specification and implementation for an ELF dynamic loader in `src/`.

# Workflow
- Read very carefully the Verus references in the reading list.
- Carefully plan the data models and spec for each stage for a project skeleton.
- Implement `src/s0_main_impl.rs` first.
- For the verified stages
  - Read the very carefully about the standard and Verus references.
  - Write a data model and formal spec for the stage in Verus in `src/s<number>_<stage_name>_spec.rs`. Keep all the spec in one file. This file shoud not contain any `external_body` or `assume_specification`.
  - Implement the stage with refinement proofs that it follows the spec in `src/s<number>_<stage_name>_impl.rs`.
- Finally, implement `src/s9_runtime.rs` to execute the final plan.
- After each stage, run proofs and test on sample input (`./run.sh --debug`).
- For debugging, use `readelf`, `objdump`, and `ldd` to inspect ELF layout, disassembly/relocations, and runtime shared-library dependencies.

# Best Practices
- Define a small mathematical spec fn first, then prove/implement against it (spec -> lemmas -> exec).
- Make sure the spec is easy for a human to read and understand with citation to the standard.
- Keep contracts minimal and semantic: requires only for real assumptions, ensures for behavior.
- Fallible stage APIs should use `Option<T>`. Use `Vec<u8>` for bytes-related data.
- Keep code simple and explicit (research prototype). Do not consider any backwards compatibility.
- DO NOT READ GIT HISTORY. DO NOT READ `third_party/musl`.

# Acceptance Checks
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
