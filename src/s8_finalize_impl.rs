use vstd::prelude::*;

use crate::s8_finalize_spec::{
    spec,
    RuntimePlan,
};
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;
use crate::s6_mmap_spec::MemoryMapPlan;
use crate::s7_reloc_spec::RelocationWrites;

verus! {

pub fn run(
    normalized: &NormalizedObjects,
    order: &DependencyOrder,
    mmap_plan: &MemoryMapPlan,
    reloc_writes: &RelocationWrites,
) -> (out: Result<RuntimePlan, String>)
    ensures
        spec(normalized, order, mmap_plan, reloc_writes, out),
{
    // TODO(stage8): assemble final entry/ctor/dtor/mapping/relocation plan.
    Err("TODO: stage8 finalize".to_string())
}

}
