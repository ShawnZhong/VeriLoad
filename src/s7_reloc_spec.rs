// Stage 7: Relocation.
// Input:
// - Parsed objects + dependency order + resolved symbols + mmap plans.
// Output:
// - Relocation write plan (target address, bytes, relocation type).
// TODO(model):
// - Add relocation write entries with target address, bytes, relocation type.
// - Link each write to requester/provider decision and addend/base inputs.
// - Keep ordering metadata for deterministic runtime application.
// TODO(spec):
// - Specify write rules for:
//   R_X86_64_RELATIVE, R_X86_64_JUMP_SLOT, R_X86_64_GLOB_DAT,
//   R_X86_64_COPY, R_X86_64_64, R_X86_64_DTPMOD64,
//   R_X86_64_DTPOFF64, R_X86_64_TPOFF64.
// - Reject unsupported relocation forms explicitly.
// - Enforce consistency with stage-5 symbol providers and stage-6 addresses.
// Standards:
// - gABI 06-reloc.rst: Relocation Entry, Relative Relocation Table.
// - gABI 08-dynamic.rst: Dynamic Section.
// - psABI object-files.tex: Relocation, Relocation Types.
// - psABI dl.tex: Dynamic Section relocation behavior.

use vstd::prelude::*;
use crate::s2_normalize_spec::NormalizedObjects;
use crate::s4_order_spec::DependencyOrder;
use crate::s5_symbol_spec::SymbolBindings;
use crate::s6_mmap_spec::MemoryMapPlan;

verus! {

#[derive(Debug)]
pub struct RelocationWrites {
}

pub open spec fn spec(
    _normalized: &NormalizedObjects,
    _order: &DependencyOrder,
    _symbols: &SymbolBindings,
    _mmap_plan: &MemoryMapPlan,
    _output: Result<RelocationWrites, String>,
) -> bool {
    true
}

}
