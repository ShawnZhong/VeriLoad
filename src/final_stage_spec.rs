use crate::mmap_plan_spec::*;
use crate::relocate_apply_spec::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn init_call_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    call: InitCall,
) -> bool {
    exists|p: int, i: int| {
        &&& 0 <= p < order.len()
        &&& (order[p] as int) < parsed.len()
        &&& 0 <= i < parsed[order[p] as int].init_array@.len()
        &&& call.pc == add_u64_or_zero(
            object_base(parsed, order, order[p] as int),
            parsed[order[p] as int].init_array@[i],
        )
    }
}

pub open spec fn term_call_sound(
    parsed: Seq<ParsedObject>,
    order: Seq<usize>,
    call: TermCall,
) -> bool {
    exists|p: int, i: int| {
        &&& 0 <= p < order.len()
        &&& (order[p] as int) < parsed.len()
        &&& 0 <= i < parsed[order[p] as int].fini_array@.len()
        &&& call.pc == add_u64_or_zero(
            object_base(parsed, order, order[p] as int),
            parsed[order[p] as int].fini_array@[i],
        )
    }
}

pub open spec fn expected_entry_pc(parsed: Seq<ParsedObject>, load_order: Seq<usize>) -> u64 {
    if parsed.len() == 0 {
        0
    } else {
        add_u64_or_zero(object_base(parsed, load_order, 0), parsed[0].entry)
    }
}

pub open spec fn final_stage_spec(in_plan: RelocateApplyOutput, out_plan: LoaderOutput) -> bool {
    &&& out_plan.entry_pc == expected_entry_pc(out_plan.parsed@, out_plan.discovered.order@)
    &&& forall|i: int|
        0 <= i < out_plan.constructors@.len() ==> init_call_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.constructors@[i],
        )
    &&& forall|i: int|
        0 <= i < out_plan.destructors@.len() ==> term_call_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.destructors@[i],
        )
    &&& out_plan.mmap_plans@ == in_plan.mmap_plans@
    &&& out_plan.reloc_writes@ == in_plan.reloc_writes@
    &&& out_plan.parsed@ == in_plan.parsed@
    &&& out_plan.discovered == in_plan.discovered
    &&& out_plan.resolved == in_plan.resolved
    &&& forall|i: int|
        0 <= i < out_plan.mmap_plans@.len() ==> mmap_plan_sound(
            out_plan.parsed@,
            out_plan.discovered.order@,
            out_plan.mmap_plans@[i],
        )
    &&& mmap_plans_non_overlapping(out_plan.mmap_plans@)
    &&& reloc_writes_sound(
        out_plan.parsed@,
        out_plan.discovered.order@,
        out_plan.resolved,
        out_plan.reloc_writes@,
    )
}

} // verus!
