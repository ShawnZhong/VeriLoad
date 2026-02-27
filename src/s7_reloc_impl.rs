use vstd::prelude::*;

use crate::s7_reloc_spec::{
    spec,
    RelocationWrites,
};
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;
use crate::s5_symbol_spec::SymbolBindings;
use crate::s6_mmap_spec::MemoryMapPlan;

verus! {

pub fn run(
    normalized: &NormalizedObjects,
    order: &DependencyOrder,
    symbols: &SymbolBindings,
    mmap_plan: &MemoryMapPlan,
) -> (out: Result<RelocationWrites, String>)
    ensures
        spec(normalized, order, symbols, mmap_plan, out),
{
    // TODO(stage7): compute relocation write plan.
    Err("TODO: stage7 reloc".to_string())
}

}
