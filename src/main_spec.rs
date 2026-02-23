use crate::discover_spec::*;
use crate::final_stage_spec::*;
use crate::mmap_plan_spec::*;
use crate::parse_spec::*;
use crate::relocate_apply_spec::*;
use crate::relocate_plan_spec::*;
use crate::resolve_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn plan_ok_spec(input: LoaderInput, out: LoaderOutput) -> bool {
    exists|
        parsed: Seq<ParsedObject>,
        discovered: DiscoveryResult,
        resolved: ResolutionResult,
        mmap_plans: Seq<MmapPlan>,
        plan_reloc: RelocatePlanOutput,
        reloc_applied: RelocateApplyOutput,
    | {
        &&& parse_stage_spec(input, parsed)
        &&& discover_stage_spec(parsed, discovered)
        &&& resolve_stage_spec(parsed, discovered, resolved)
        &&& mmap_plan_stage_spec(parsed, discovered, mmap_plans)
        &&& plan_relocate_stage_spec(parsed, discovered, resolved, mmap_plans, plan_reloc)
        &&& relocate_apply_stage_spec(plan_reloc, reloc_applied)
        &&& final_stage_spec(reloc_applied, out)
    }
}

pub open spec fn plan_result_spec(input: LoaderInput, out: Result<LoaderOutput, LoaderError>) -> bool {
    match out {
        Ok(plan) => plan_ok_spec(input, plan),
        Err(_) => true,
    }
}

} // verus!
