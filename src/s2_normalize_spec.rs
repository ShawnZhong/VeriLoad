// Stage 2: Normalization.
// Input:
// - List of RawObjects.
// Output:
// - Canonical parsed objects with semantic checks and normalized metadata
//   suitable for dependency/symbol/mapping/relocation/finalization stages.
// TODO(model):
// - Add canonical object metadata needed by later stages.
// - Add normalized string/name views used for dependency matching.
// - Add normalized symbol tables and relocation views.
// - Add normalized init/fini metadata used by stage 8.
// TODO(spec):
// - Specify semantic checks required before dependency and symbol stages.
// - Require internal consistency across names/symbols/relocations.
// - Require rejected inputs to return Err (no partial normalized state).
// Standards:
// - gABI 02-eheader.rst: Contents of the ELF Header, ELF Identification.
// - gABI 04-strtab.rst: String Table.
// - gABI 05-symtab.rst: Symbol Table, Symbol Table Entry, Symbol Binding,
//   Symbol Type, Symbol Visibility, Section Index.
// - gABI 06-reloc.rst: Relocation Entry.
// - gABI 08-dynamic.rst: Dynamic Section.
// - psABI object-files.tex: ELF Header, Symbol Table, Relocation.

use vstd::prelude::*;
use crate::s1_parse_spec::RawObjects;

verus! {

#[derive(Debug)]
pub struct NormalizedObjects {
}

pub open spec fn spec(_parsed: &RawObjects, _output: Result<NormalizedObjects, String>) -> bool {
    true
}

}
