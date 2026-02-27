use vstd::prelude::*;

use crate::s5_symbol_spec::{
    spec,
    SymbolBindings,
};
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;

verus! {

pub fn run(normalized: &NormalizedObjects, order: &DependencyOrder) -> (out: Result<SymbolBindings, String>)
    ensures
        spec(normalized, order, out),
{
    // TODO(stage5): resolve relocation symbols against dependency scope.
    Err("TODO: stage5 symbol".to_string())
}

}
