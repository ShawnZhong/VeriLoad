# TODO

1. Implement executable parser that produces `ParseStageState` and proves `parse_stage_ok`.
2. Implement `DT_NEEDED` resolver over provided object list and prove `dependency_stage_ok`.
3. Implement load-bias allocator and segment image builder and prove `layout_stage_ok`.
4. Implement symbol lookup engine (BFS + weak/global + `DT_SYMBOLIC`) and prove `symbol_stage_ok`.
5. Implement relocation evaluator for supported machines/types and prove `relocation_stage_ok`.
6. Implement initializer ordering (dependencies first, single-call guarantee) and prove `init_stage_ok`.
7. Implement final exporter to `LoaderOutput` and prove `finalize_stage_ok`.
8. Add unsupported-relocation/object-format fatal paths (panic loudly).
9. Add end-to-end theorem/lemma that implementation result satisfies `loader_spec`.
10. Add focused regression cases:
    - single DSO,
    - multi-DSO BFS symbol resolution,
    - missing `DT_NEEDED`,
    - weak unresolved symbol,
    - `DT_SYMBOLIC`,
    - `DT_BIND_NOW`/eager relocation behavior.
