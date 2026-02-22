## Objective
Based on the ELF documentation, derive and implement a formal Verus specification for an ELF dynamic loader in `src/`.
The resulting binary should be able to load and run the `build/main` executable.

## Start Here
1. Reading List
    - ELF gABI specification: `third_party/gABI/docsrc/elf`. Focus on the following sections:
        - 02-eheader.rst
        - 03-sheader.rst
        - 06-reloc.rst
        - 07-pheader.rst
        - 08-dynamic.rst
    - x86_84 psABI: `third_party/psABI_x86_64/x86-64-ABI`. Focus on the following sections:
        - object-files.tex
    - Verus examples: `third_party/verus/examples`. Focus on the following files:
        - `examples/guide/recursion.rs`
        - `source/docs/guide/src/triangle.md` and 
2. Plan the design in `design.md` and the todo list in `todo.md` first.
3. Stage by stage, come up with the spec/struct for each stage first. Then implement, and most importantly, prove the implementation refines the spec.

## LoaderInput
- `LoaderInput` is provided by unverified initialization code calling into the verified code.
- `LoaderInput` contains a sequence of objects; each object has a name and raw bytes.
- Assume that all shared libraries are placed at the same directory as the executable.

## LoaderOutput
`LoaderOutput` must include:
- entry PC
- ordered list of constructors to call
- ordered list of destructors to call
- sequence of `mmap` plans

Each `mmap` plan must include:
- starting address
- bytes to map
- protection flags

## Staged Modeling Requirements
- Organize the loader into explicit stages, with spec/struct per stage when useful.
- For each stage, write spec in `src/<stage_name>_spec.rs` and implementation in `src/<stage_name>_impl.rs`. 
- Put all the structs and constants common to all stages in `src/types.rs` and `src/consts.rs`.
- Suggested stages
    - parse binary
    - build dependency graph
    - order dependencies
    - plan memory mapping locations
    - resolve symbols
    - plan relocations
    - apply relocations
    - finalize the output
- Keep the design easy to implement and verify.
- Do not omit details required for a correct loader specification.
- Since input is bytes, model all required ELF constants from the spec.
- DO NOT CHEAT!!! external_body is only allowed in the verification boundary.
- Make sure that you prove that the implementation refines the spec. Do not skip any steps.

## Implementation and Proof Requirements
- Ensure implementation behavior matches the formal spec.
- Prove that the implementation satisfies the spec, stage by stage and end-to-end.
- For each implemented loader stage, include explicit refinement/correctness lemmas that connect implementation results to spec results.
- Fully prove spec/implementation correspondence; no proof shortcuts.
- Do not read git history.
- Do not put everything in one file.
- Use `tests/` to test the loader.

## Verification Boundary
The following may remain unverified, but you still need to implement them:
- initialization code that invokes verified loader with `LoaderInput`
- runtime code that consumes `LoaderOutput`

## Initialization Code Responsibilities
- read the current executable and all shared libraries (.so files) from the same directory from the filesystem.
- build `LoaderInput`.
- invoke the verified loader.

## Runtime Responsibilities
Given `LoaderOutput`, runtime code must:
- perform segment `mmap`s
- call constructors in order
- set up stack state
- jump to the entry point: you will need to write some assembly code to do this.

## Testing
- Test dependency discovery order.
- Test symbol resolution and relocation results.
- Test that generated `mmap` plans match expected addresses, bytes, and flags.
- Test initializer call order.
- Run Verus proofs and test commands on every change.

## Final Deliverable
- `design.md` with the full design.
- `todo.md` with staged implementation plan and progress.
- Verified loader spec and model code in `src/`.
- Implemented and tested initialization and runtime code at the verification boundary.
