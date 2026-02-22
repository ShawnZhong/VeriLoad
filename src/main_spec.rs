use crate::discover_spec::*;
use crate::parse_spec::*;
use crate::relocate_spec::*;
use crate::resolve_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn plan_ok_spec(input: LoaderInput, out: LoaderOutput) -> bool {
    exists|parsed: Seq<ParsedObject>, discovered: DiscoveryResult, resolved: ResolutionResult| {
        &&& parse_stage_spec(input, parsed)
        &&& discover_stage_spec(parsed, discovered)
        &&& resolve_stage_spec(parsed, discovered, resolved)
        &&& relocate_stage_spec(parsed, discovered, resolved, out)
    }
}

pub open spec fn plan_result_spec(input: LoaderInput, out: Result<LoaderOutput, LoaderError>) -> bool {
    match out {
        Ok(plan) => plan_ok_spec(input, plan),
        Err(_) => true,
    }
}

} // verus!
