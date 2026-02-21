## Objective
Based on `docs/elf.md` and `docs/x86_64.md`, derive and implement a formal Verus specification for an ELF dynamic loader in `src/`.
The result should be suitable for a real implementation. Use `verus/examples` as style guidance.

## Start Here
1. Write `design.md` and `todo.md` first.
2. At the beginning, implement the core interface types (`LoaderInput`, `LoaderOutput`).
3. Then implement and prove the loader stage by stage. Do not implement everything at once.

## LoaderInput
- `LoaderInput` is provided by unverified code calling into the loader.
- `LoaderInput` contains a sequence of objects; each object has a name and raw bytes.
- Assume `/lib` is the only directory used to search for `.so` files.

## LoaderOutput
`LoaderOutput` must include:
- entry PC
- ordered list of initializers to call
- sequence of `mmap` plans

Each `mmap` plan must include:
- starting address
- bytes to map
- protection flags

## Staged Modeling Requirements
- Organize the loader into explicit stages, with spec/struct per stage when useful.
- Suggested stages: parse, dependency discovery, symbol resolution, relocation.
- Keep the design easy to implement and verify.
- Do not omit details required for a correct loader specification.
- Since input is bytes, model all required ELF constants from the spec.

## Implementation and Proof Requirements
- Ensure implementation behavior matches the formal spec.
- Fully prove spec/implementation correspondence; no proof shortcuts.
- Do not read git history.

## Verification Boundary
The following may remain unverified, but you still need to implement them:
- initialization code that invokes verified loader with `LoaderInput`
- runtime code that consumes `LoaderOutput`

## Initialization Code Responsibilities
- read files from the filesystem.
- build `LoaderInput`.
- invoke the verified loader.

## Runtime Responsibilities
Given `LoaderOutput`, runtime code must:
- perform segment `mmap`s
- call initializers in order
- set up stack state
- jump to the entry point

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
