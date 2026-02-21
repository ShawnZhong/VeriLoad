# VeriLoad Dynamic Loader Spec Design

This prototype defines a formal, stage-based ELF dynamic loader spec in Verus.

## Requested Interface

- Input: an ordered sequence of objects.
- Each object: `(name, bytes)`.
- Output:
  - `entry_pc`: final program entry address.
  - `initializers`: ordered list of initializer PCs to call.
  - `mmap_plans`: ordered mapping plans, each with:
    - start address,
    - bytes image to map,
    - protection flags.

## Stage Decomposition

The loader is decomposed into explicit stages so implementation and verification can align:

1. Parse Stage
   - Parse ELF headers/program headers/dynamic tables/symbols/relocations.
   - Validate required ELF invariants.
2. Dependency Discovery Stage
   - Resolve `DT_NEEDED` edges.
   - Build BFS load order and dependency graph.
3. Layout Stage
   - Assign load bias/base addresses.
   - Build per-segment map images from `PT_LOAD`.
4. Symbol Resolution Stage
   - Resolve relocation symbols with BFS scope.
   - Enforce weak/global precedence and `DT_SYMBOLIC` behavior.
5. Relocation Stage
   - Compute relocation writes from architecture formulas.
   - Produce relocated map images.
6. Initialization Stage
   - Produce dependency-respecting initializer order.
7. Finalization
   - Export required output tuple: `entry_pc`, `initializers`, `mmap_plans`.

## Files

- `src/model.rs`: all formal data models (input, output, ELF entities, and per-stage states).
- `src/spec.rs`: formal stage predicates plus `loader_spec` composition predicate.

## ELF Constants Modeling

Because object input is raw bytes, `src/model.rs` includes ELF numeric constants from the spec:

- `e_ident` indexes and magic/class/data/version values.
- `e_type`, `e_machine`, `PT_*`, `PF_*`.
- `DT_*`, symbol binding/type constants, and key section-index constants.
- dynamic relocation type IDs for i386/x86_64 used by runtime relocation logic.
- canonical entry sizes for ELF32/ELF64 headers/tables used by parsers.

The model also includes decode helpers (numeric field -> enum) so parser code can map bytes to
the formal model directly.
