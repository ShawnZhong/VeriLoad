use vstd::prelude::*;

use crate::s6_mmap_spec::{
    spec,
    MemoryMapPlan,
};
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;

verus! {

pub fn run(normalized: &NormalizedObjects, order: &DependencyOrder) -> (out: Result<MemoryMapPlan, String>)
    ensures
        spec(normalized, order, out),
{
    // TODO(stage6): plan mmap regions and base addresses.
    Err("TODO: stage6 mmap".to_string())
}

}
