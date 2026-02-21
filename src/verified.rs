use vstd::prelude::*;

use crate::model::*;
use crate::spec::*;

verus! {

pub proof fn stage_pipeline_implies_loader_spec(
    input: LoaderInput,
    parse: ParseStageState,
    dep: DependencyStageState,
    layout: LayoutStageState,
    sym_stage: SymbolResolutionStageState,
    reloc_stage: RelocationStageState,
    init_stage: InitStageState,
    out: LoaderOutput,
)
    requires
        parse_stage_ok(input, parse),
        dependency_stage_ok(parse, dep),
        layout_stage_ok(parse, dep, layout, input.page_size),
        symbol_stage_ok(parse, dep, layout, sym_stage),
        relocation_stage_ok(parse, dep, layout, sym_stage, reloc_stage),
        init_stage_ok(parse, dep, layout, init_stage),
        finalize_stage_ok(parse, dep, layout, reloc_stage, init_stage, out),
    ensures
        loader_spec(input, out),
{
    assert(loader_spec(input, out));
}

} // verus!
