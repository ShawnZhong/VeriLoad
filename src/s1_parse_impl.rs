use vstd::prelude::*;

use crate::s0_main_spec::PlannerInput;
use crate::s1_parse_spec::{spec, RawObjects};

verus! {

pub fn run(input: PlannerInput) -> (out: Result<RawObjects, String>)
    ensures
        spec(input, out),
{
    // TODO(stage1): parse bytes into raw ELF header/table structures.
    Err("TODO: stage1 parse".to_string())
}

}
