mod elf;
mod model;
mod planner;
mod rt_common;
mod runtime;
mod spec;
mod verified;

use vstd::prelude::*;
use model::*;
use spec::*;

verus! {

#[verifier::external_body]
fn load_and_plan_unverified() -> (plan: rt_common::RuntimePlan)
    ensures
        exists|input: LoaderInput, out: LoaderOutput|
            loader_spec(input, out) && runtime_output_matches_spec(plan.stage.output, out),
{
    runtime::load_and_plan_unverified_impl()
}

#[verifier::external_body]
fn runtime_handoff_unverified(plan: rt_common::RuntimePlan) -> ! {
    runtime::runtime_handoff_unverified_impl(plan)
}

pub open spec fn perm_matches_spec(
    rt: rt_common::SegmentPerm,
    m: SegmentPerm,
) -> bool {
    &&& rt.read == m.read
    &&& rt.write == m.write
    &&& rt.execute == m.execute
}

pub open spec fn init_call_matches_spec(
    rt: rt_common::InitializerCall,
    m: InitializerCall,
) -> bool {
    &&& rt.object_id as nat == m.object_id
    &&& rt.pc as nat == m.pc
}

pub open spec fn mmap_plan_matches_spec(
    rt: rt_common::SegmentMapPlan,
    m: MmapPlan,
) -> bool {
    &&& rt.start as nat == m.start
    &&& rt.bytes@ == m.bytes
    &&& perm_matches_spec(rt.prot, m.prot)
}

pub open spec fn runtime_output_matches_spec(
    rt: rt_common::LoaderOutput,
    m: LoaderOutput,
) -> bool {
    &&& rt.entry_pc as nat == m.entry_pc
    &&& rt.initializers.len() == m.initializers.len()
    &&& forall|i: int|
        0 <= i < rt.initializers.len()
            ==> init_call_matches_spec(rt.initializers[i], m.initializers[i])
    &&& rt.mmap_plans.len() == m.mmap_plans.len()
    &&& forall|i: int|
        0 <= i < rt.mmap_plans.len()
            ==> mmap_plan_matches_spec(rt.mmap_plans[i], m.mmap_plans[i])
}

fn main() {
    let plan = load_and_plan_unverified();
    runtime_handoff_unverified(plan);
}

} // verus!
