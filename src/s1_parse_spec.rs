// Stage 1: Byte Decode.
// Input:
// - List of BinaryObjects.
// Output:
// - Raw decoded ELF structures (headers/tables/offsets/relocations/dynamic entries)
// TODO(model):
// - Add per-object decoded ELF header fields needed by later stages.
// - Add decoded program headers (PT_LOAD/PT_DYNAMIC and flags).
// - Add decoded dynamic entries for needed tags and relocation tags.
// - Add decoded relocation tables (RELA/JMPREL and RELR when present).
// - Keep offset/size metadata needed for bounds proofs.
// TODO(spec):
// - Reject malformed offsets/sizes/alignment with Err.
// - Define well-formedness predicates for ELF headers/program headers.
// - Define decode-success contract for dynamic and relocation tables.
// - Include class/endianness/machine acceptance checks.
// Standards:
// - gABI 02-eheader.rst: ELF Header, Contents of the ELF Header,
//   ELF Identification, Data Encoding.
// - gABI 07-pheader.rst: Program Header Entry, Segment Types.
// - gABI 08-dynamic.rst: Dynamic Section.
// - gABI 06-reloc.rst: Relocation Entry, Relative Relocation Table.
// - psABI object-files.tex: ELF Header, Machine Information,
//   Number of Program Headers.

use vstd::prelude::*;
use crate::s0_main_spec::PlannerInput;

verus! {

#[derive(Debug)]
pub struct RawObjects {
}

pub open spec fn spec(_input: PlannerInput, _output: Result<RawObjects, String>) -> bool {
    true
}

}
