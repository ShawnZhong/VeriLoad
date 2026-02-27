// Stage 5: Symbol Resolution.
// Input:
// - Parsed objects + dependency order.
// Output:
// - Provider decision for each relevant relocation.
// TODO(model):
// - Add provider decision per relocation.
// - Represent unresolved outcomes for ABI-allowed weak-undefined symbols.
// - Keep metadata needed by stage-7 relocation write planning.
// TODO(spec):
// - Specify provider search order over stage-4 dependency order.
// - Specify strong/weak precedence and visibility constraints.
// - Specify unresolved-provider errors for required relocations.
// Standards:
// - gABI 06-reloc.rst: Relocation Entry.
// - gABI 08-dynamic.rst: Dynamic Section, Hash Table.
// - psABI object-files.tex: Symbol Table, Relocation, Relocation Types.
// - psABI dl.tex: Dynamic Section lookup behavior.

use vstd::prelude::*;
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;

verus! {

#[derive(Debug)]
pub struct SymbolBindings {
}

pub open spec fn spec(
    _normalized: &NormalizedObjects,
    _order: &DependencyOrder,
    _output: Result<SymbolBindings, String>,
) -> bool {
    true
}

}
