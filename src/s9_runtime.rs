// TODO(stage9):
// - Execute plan in order: map -> apply relocation writes -> protect.
// - Build initial stack/auxv/argv image using forwarded CLI args.
// - Include at least AT_PAGESZ, AT_BASE, AT_ENTRY, AT_RANDOM, AT_EXECFN,
//   and AT_PHDR/AT_PHENT/AT_PHNUM when available.
// - Call constructors, transfer control to entry PC, then run destructors.
//
// Standards to reference while implementing runtime behavior:
// - psABI low-level-sys-info.tex: Process Initialization,
//   Initial Stack and Register State, Stack State, Auxiliary Vector.
// - gABI 02-eheader.rst: Contents of the ELF Header.
// - gABI 07-pheader.rst: Program Header Entry, Segment Types.
// - gABI 08-dynamic.rst: Initialization and Termination Functions.

use crate::s8_finalize_spec::RuntimePlan;

pub fn run_runtime(_plan: RuntimePlan, _forwarded_args: Vec<String>) -> Result<(), String> {
    Err("TODO: stage9 runtime execution".to_string())
}
